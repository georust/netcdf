//! Open, create, and append netcdf files
#![allow(clippy::similar_names)]

use std::marker::PhantomData;
use std::path;

use netcdf_sys::*;

use super::attribute::{Attribute, AttributeValue};
use super::dimension::{self, Dimension};
use super::error;
use super::group::{Group, GroupMut};
use super::types::{NcTypeDescriptor, NcVariableType};
use super::variable::{Variable, VariableMut};
use crate::group::{get_parent_ncid_and_stem, try_get_ncid, try_get_parent_ncid_and_stem};
use crate::utils::with_lock;

#[derive(Debug)]
#[repr(transparent)]
pub(crate) struct RawFile {
    ncid: nc_type,
}

impl RawFile {
    fn close(self) -> error::Result<()> {
        let Self { ncid } = self;
        error::checked(with_lock(|| unsafe { nc_close(ncid) }))
    }
}

impl Drop for RawFile {
    fn drop(&mut self) {
        // Can't really do much with an error here
        let ncid = self.ncid;
        let _err = error::checked(with_lock(|| unsafe { nc_close(ncid) }));
    }
}

#[cfg(unix)]
fn get_ffi_from_path(path: &path::Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    let mut bytes = path.as_os_str().as_bytes().to_vec();
    bytes.push(0);
    bytes
}
#[cfg(not(unix))]
fn get_ffi_from_path(path: &path::Path) -> std::ffi::CString {
    std::ffi::CString::new(path.to_str().unwrap()).unwrap()
}

bitflags::bitflags! {
    /// Options for opening, creating, and appending files
    #[derive(Default)]
    pub struct Options: nc_type {
        /// Open with write permissions (use `append` for a mutable file)
        const WRITE = NC_WRITE;
        /// Overwrite existing file
        const NOCLOBBER = NC_NOCLOBBER;
        /// Reads file into memory
        const DISKLESS = NC_DISKLESS;
        /// Use 64 bit dimensions and sizes (`CDF-5` format)
        const _64BIT_DATA = NC_64BIT_DATA;
        /// Use 64 bit file offsets
        const _64BIT_OFFSET = NC_64BIT_OFFSET;
        /// Use a subset compatible with older software
        const CLASSIC = NC_CLASSIC_MODEL;
        /// Limits internal caching
        const SHARE = NC_SHARE;
        /// Use the `hdf5` storage format
        const NETCDF4 = NC_NETCDF4;
        /// Read from memory
        const INMEMORY = NC_INMEMORY;
    }
}

impl RawFile {
    /// Open a `netCDF` file in read only mode.
    pub(crate) fn open_with(path: &path::Path, options: Options) -> error::Result<File> {
        let f = get_ffi_from_path(path);
        let mut ncid: nc_type = 0;
        unsafe {
            error::checked(with_lock(|| {
                nc_open(f.as_ptr().cast(), options.bits(), &mut ncid)
            }))?;
        }
        Ok(File(Self { ncid }))
    }

    /// Open a `netCDF` file in read only mode in parallel mode.
    #[cfg(feature = "mpi")]
    pub(crate) fn open_par_with(
        path: &path::Path,
        communicator: mpi_sys::MPI_Comm,
        info: mpi_sys::MPI_Info,
        options: Options,
    ) -> error::Result<File> {
        let f = get_ffi_from_path(path);
        let mut ncid: nc_type = 0;
        unsafe {
            error::checked(super::with_lock(|| {
                netcdf_sys::par::nc_open_par(
                    f.as_ptr().cast(),
                    options.bits(),
                    communicator,
                    info,
                    &mut ncid,
                )
            }))?;
        }
        Ok(File(Self { ncid }))
    }

    /// Open a `netCDF` file in append mode (read/write).
    pub(crate) fn append_with(path: &path::Path, options: Options) -> error::Result<FileMut> {
        let file = Self::open_with(path, options | Options::WRITE)?;
        Ok(FileMut(file))
    }

    /// Create a new `netCDF` file
    pub(crate) fn create_with(path: &path::Path, options: Options) -> error::Result<FileMut> {
        let f = get_ffi_from_path(path);
        let mut ncid: nc_type = -1;
        unsafe {
            error::checked(with_lock(|| {
                nc_create(f.as_ptr().cast(), options.bits(), &mut ncid)
            }))?;
        }

        Ok(FileMut(File(Self { ncid })))
    }

    /// Create a new `netCDF` file in parallel mode
    #[cfg(feature = "mpi")]
    pub(crate) fn create_par_with(
        path: &path::Path,
        communicator: mpi_sys::MPI_Comm,
        info: mpi_sys::MPI_Info,
        options: Options,
    ) -> error::Result<FileMut> {
        let f = get_ffi_from_path(path);
        let mut ncid: nc_type = -1;
        unsafe {
            error::checked(super::with_lock(|| {
                netcdf_sys::par::nc_create_par(
                    f.as_ptr().cast(),
                    options.bits(),
                    communicator,
                    info,
                    &mut ncid,
                )
            }))?;
        }

        Ok(FileMut(File(Self { ncid })))
    }

    #[cfg(feature = "has-mmap")]
    pub(crate) fn open_from_memory<'buffer>(
        name: Option<&str>,
        mem: &'buffer [u8],
    ) -> error::Result<FileMem<'buffer>> {
        let cstr = std::ffi::CString::new(name.unwrap_or("/")).unwrap();
        let mut ncid = 0;
        unsafe {
            error::checked(with_lock(|| {
                nc_open_mem(
                    cstr.as_ptr(),
                    NC_NOWRITE,
                    mem.len(),
                    mem.as_ptr().cast_mut().cast(),
                    &mut ncid,
                )
            }))?;
        }

        Ok(FileMem(File(Self { ncid }), PhantomData))
    }
}

#[derive(Debug)]
/// Read only accessible file
#[allow(clippy::module_name_repetitions)]
#[repr(transparent)]
pub struct File(RawFile);

impl File {
    /// path used to open/create the file
    ///
    /// #Errors
    ///
    /// Netcdf layer could fail, or the resulting path
    /// could contain an invalid UTF8 sequence
    pub fn path(&self) -> error::Result<std::path::PathBuf> {
        let name: Vec<u8> = {
            let mut pathlen = 0;
            unsafe {
                error::checked(with_lock(|| {
                    nc_inq_path(self.0.ncid, &mut pathlen, std::ptr::null_mut())
                }))?;
            }
            let mut name = vec![0_u8; pathlen + 1];
            unsafe {
                error::checked(with_lock(|| {
                    nc_inq_path(self.0.ncid, std::ptr::null_mut(), name.as_mut_ptr().cast())
                }))?;
            }
            name.truncate(pathlen);
            name
        };

        #[cfg(not(unix))]
        {
            Ok(std::path::PathBuf::from(String::from_utf8(name)?))
        }
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            let osstr = std::ffi::OsStr::from_bytes(&name);
            Ok(std::path::PathBuf::from(osstr))
        }
    }

    /// Main entrypoint for interacting with the netcdf file.
    pub fn root(&self) -> Option<Group> {
        let mut format = 0;
        unsafe { error::checked(with_lock(|| nc_inq_format(self.ncid(), &mut format))) }.unwrap();

        match format {
            NC_FORMAT_NETCDF4 | NC_FORMAT_NETCDF4_CLASSIC => Some(Group {
                ncid: self.ncid(),
                _file: PhantomData,
            }),
            _ => None,
        }
    }

    fn ncid(&self) -> nc_type {
        self.0.ncid
    }

    /// Get a variable from the group
    pub fn variable<'f>(&'f self, name: &str) -> Option<Variable<'f>> {
        let (ncid, name) =
            super::group::try_get_parent_ncid_and_stem(self.ncid(), name).unwrap()?;
        Variable::find_from_name(ncid, name).unwrap()
    }
    /// Iterate over all variables in a group
    pub fn variables(&self) -> impl Iterator<Item = Variable> {
        super::variable::variables_at_ncid(self.ncid())
            .unwrap()
            .map(Result::unwrap)
    }
    /// Get a single attribute
    pub fn attribute<'f>(&'f self, name: &str) -> Option<Attribute<'f>> {
        let (ncid, name) = try_get_parent_ncid_and_stem(self.ncid(), name).unwrap()?;
        Attribute::find_from_name(ncid, None, name).unwrap()
    }
    /// Get all attributes in the root group
    pub fn attributes(&self) -> impl Iterator<Item = Attribute> {
        crate::attribute::AttributeIterator::new(self.0.ncid, None)
            .unwrap()
            .map(Result::unwrap)
    }

    /// Get a single dimension
    pub fn dimension<'f>(&self, name: &str) -> Option<Dimension<'f>> {
        let (ncid, name) =
            super::group::try_get_parent_ncid_and_stem(self.ncid(), name).unwrap()?;
        super::dimension::dimension_from_name(ncid, name).unwrap()
    }
    /// Iterator over all dimensions in the root group
    pub fn dimensions(&self) -> impl Iterator<Item = Dimension> {
        super::dimension::dimensions_from_location(self.ncid())
            .unwrap()
            .map(Result::unwrap)
    }
    /// Get the length of a dimension
    pub fn dimension_len<'f>(&self, name: &str) -> Option<usize> {
        let (ncid, name) =
            super::group::try_get_parent_ncid_and_stem(self.ncid(), name).unwrap()?;
        super::dimension::dimension_from_name(ncid, name)
            .unwrap()
            .map(|x| x.len())
    }

    /// Get a group
    ///
    /// # Errors
    ///
    /// Not a `netCDF-4` file
    pub fn group<'f>(&'f self, name: &str) -> error::Result<Option<Group<'f>>> {
        let (ncid, name) = get_parent_ncid_and_stem(self.ncid(), name)?;
        try_get_ncid(ncid, name).map(|ncid: Option<i32>| {
            ncid.map(|ncid| Group {
                ncid,
                _file: PhantomData,
            })
        })
    }
    /// Iterator over all subgroups in the root group
    ///
    /// # Errors
    ///
    /// Not a `netCDF-4` file
    pub fn groups(&self) -> error::Result<impl Iterator<Item = Group>> {
        super::group::groups_at_ncid(self.ncid())
    }

    /// Return all types in the root group
    pub fn types(&self) -> error::Result<impl Iterator<Item = NcVariableType>> {
        super::types::all_at_location(self.ncid()).map(|x| x.map(Result::unwrap))
    }

    /// Close the file
    ///
    /// Note: This is called automatically by `Drop`, but can be useful
    /// if flushing data or closing the file would result in an error.
    pub fn close(self) -> error::Result<()> {
        let Self(file) = self;
        file.close()
    }

    /// Access all variable in independent mode
    /// for parallell reading using MPI.
    /// File must have been opened using `open_par`
    ///
    /// This is the default access mode
    #[cfg(feature = "mpi")]
    pub fn access_independent(&self) -> error::Result<()> {
        let ncid = self.ncid();
        crate::par::set_access_mode(
            ncid,
            netcdf_sys::NC_GLOBAL,
            crate::par::AccessMode::Independent,
        )
    }
    /// Access all variable in collective mode
    /// for parallell reading using MPI.
    /// File must have been opened using `open_par`
    #[cfg(feature = "mpi")]
    pub fn access_collective(&self) -> error::Result<()> {
        let ncid = self.ncid();
        crate::par::set_access_mode(
            ncid,
            netcdf_sys::NC_GLOBAL,
            crate::par::AccessMode::Collective,
        )
    }
}

/// Mutable access to file.
///
/// This type derefs to a [`File`](File), which means [`FileMut`](Self)
/// can be used where [`File`](File) is expected
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
#[repr(transparent)]
pub struct FileMut(File);

impl std::ops::Deref for FileMut {
    type Target = File;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FileMut {
    /// Mutable access to the root group
    ///
    /// Return None if this can't be a root group
    pub fn root_mut(&mut self) -> Option<GroupMut> {
        self.root().map(|root| GroupMut(root, PhantomData))
    }
    /// Get a mutable variable from the group
    pub fn variable_mut<'f>(&'f mut self, name: &str) -> Option<VariableMut<'f>> {
        self.variable(name).map(|var| VariableMut(var, PhantomData))
    }
    /// Iterate over all variables in the root group, with mutable access
    ///
    /// # Examples
    /// Use this to get multiple writable variables
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut file = netcdf::append("file.nc")?;
    /// let mut vars = file.variables_mut().collect::<Vec<_>>();
    /// vars[0].put_value(1_u8, [2, 5])?;
    /// vars[1].put_value(1_u8, [5, 2])?;
    /// # Ok(()) }
    /// ```
    pub fn variables_mut(&mut self) -> impl Iterator<Item = VariableMut> {
        self.variables().map(|var| VariableMut(var, PhantomData))
    }

    /// Mutable access to subgroup
    ///
    /// # Errors
    ///
    /// File does not support groups
    pub fn group_mut<'f>(&'f mut self, name: &str) -> error::Result<Option<GroupMut<'f>>> {
        self.group(name)
            .map(|g| g.map(|g| GroupMut(g, PhantomData)))
    }
    /// Iterator over all groups (mutable access)
    ///
    /// # Errors
    ///
    /// File does not support groups
    pub fn groups_mut(&mut self) -> error::Result<impl Iterator<Item = GroupMut>> {
        self.groups().map(|g| g.map(|g| GroupMut(g, PhantomData)))
    }

    /// Add an attribute to the root group
    pub fn add_attribute<'a, T>(&'a mut self, name: &str, val: T) -> error::Result<Attribute<'a>>
    where
        T: Into<AttributeValue>,
    {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.ncid(), name)?;
        Attribute::put(ncid, NC_GLOBAL, name, val.into())
    }

    /// Adds a dimension with the given name and size. A size of zero gives an unlimited dimension
    pub fn add_dimension<'f>(&'f mut self, name: &str, len: usize) -> error::Result<Dimension<'f>> {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.ncid(), name)?;
        super::dimension::add_dimension_at(ncid, name, len)
    }
    /// Adds a dimension with unbounded size
    pub fn add_unlimited_dimension(&mut self, name: &str) -> error::Result<Dimension> {
        self.add_dimension(name, 0)
    }

    /// Add an empty group to the dataset
    pub fn add_group<'f>(&'f mut self, name: &str) -> error::Result<GroupMut<'f>> {
        Ok(GroupMut(
            Group {
                ncid: super::group::add_group_at_path(self.ncid(), name)?,
                _file: PhantomData,
            },
            PhantomData,
        ))
    }

    /// Create a Variable into the dataset, with no data written into it
    ///
    /// Dimensions are identified using the name of the dimension, and will recurse upwards
    /// if not found in the current group.
    pub fn add_variable<'f, T>(
        &'f mut self,
        name: &str,
        dims: &[&str],
    ) -> error::Result<VariableMut<'f>>
    where
        T: NcTypeDescriptor,
    {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.ncid(), name)?;
        VariableMut::add_from_str(ncid, &T::type_descriptor(), name, dims)
    }

    /// Create a variable with the specified type
    pub fn add_variable_with_type<'f>(
        &'f mut self,
        name: &str,
        dims: &[&str],
        typ: &NcVariableType,
    ) -> error::Result<VariableMut<'f>> {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.ncid(), name)?;
        VariableMut::add_from_str(ncid, typ, name, dims)
    }

    /// Add a type to the file
    /// Usually the file is `derive`'d using `NcType`
    pub fn add_type<T: NcTypeDescriptor>(&mut self) -> error::Result<nc_type> {
        crate::types::add_type(self.ncid(), T::type_descriptor(), false)
    }
    /// Add a type using a descriptor
    pub fn add_type_from_descriptor(&mut self, typ: NcVariableType) -> error::Result<nc_type> {
        crate::types::add_type(self.ncid(), typ, false)
    }

    /// Adds a variable from a set of unique identifiers, recursing upwards
    /// from the current group if necessary.
    pub fn add_variable_from_identifiers<'f, T>(
        &'f mut self,
        name: &str,
        dims: &[dimension::DimensionIdentifier],
    ) -> error::Result<VariableMut<'f>>
    where
        T: NcTypeDescriptor,
    {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.ncid(), name)?;
        let Some(xtype) = super::types::find_type(ncid, &T::type_descriptor())? else {
            return Err("Type not found at this location".into());
        };
        super::variable::add_variable_from_identifiers(ncid, name, dims, xtype)
    }
    /// Adds a variable from a set of unique identifiers, recursing upwards
    /// from the current group if necessary.
    pub fn add_variable_from_identifiers_with_type<'f>(
        &'f mut self,
        name: &str,
        dims: &[dimension::DimensionIdentifier],
        typ: &NcVariableType,
    ) -> error::Result<VariableMut<'f>> {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.ncid(), name)?;
        let Some(xtype) = super::types::find_type(ncid, typ)? else {
            return Err("Type not found at this location".into());
        };
        super::variable::add_variable_from_identifiers(ncid, name, dims, xtype)
    }

    /// Flush pending buffers to disk to minimise data loss in case of termination.
    ///
    /// Note: When writing and reading from the same file from multiple processes
    /// it is recommended to instead open the file in both the reader and
    /// writer process with the [`Options::SHARE`] flag.
    pub fn sync(&self) -> error::Result<()> {
        error::checked(with_lock(|| unsafe { netcdf_sys::nc_sync(self.ncid()) }))
    }

    /// Close the file
    ///
    /// Note: This is called automatically by `Drop`, but can be useful
    /// if flushing data or closing the file would result in an error.
    pub fn close(self) -> error::Result<()> {
        let Self(File(file)) = self;
        file.close()
    }

    /// Open the file for new definitions
    pub fn redef(&mut self) -> error::Result<()> {
        error::checked(super::with_lock(|| unsafe {
            netcdf_sys::nc_redef(self.ncid())
        }))
    }

    /// Close the file for new definitions
    pub fn enddef(&mut self) -> error::Result<()> {
        error::checked(super::with_lock(|| unsafe {
            netcdf_sys::nc_enddef(self.ncid())
        }))
    }
}

#[cfg(feature = "has-mmap")]
/// The memory mapped file is kept in this structure to extend
/// the lifetime of the buffer.
///
/// Access a [`File`] through the `Deref` trait,
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let buffer = &[0, 42, 1, 2];
/// let file = &netcdf::open_mem(None, buffer)?;
///
/// let variables = file.variables();
/// # Ok(()) }
/// ```
#[allow(clippy::module_name_repetitions)]
pub struct FileMem<'buffer>(File, std::marker::PhantomData<&'buffer [u8]>);

#[cfg(feature = "has-mmap")]
impl<'a> std::ops::Deref for FileMem<'a> {
    type Target = File;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
