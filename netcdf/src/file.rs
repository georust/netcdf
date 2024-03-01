//! Open, create, and append netcdf files
#![allow(clippy::similar_names)]

use std::marker::PhantomData;
use std::path;

use netcdf_sys::*;

use super::attribute::{Attribute, AttributeValue};
use super::dimension::{AsNcDimensions, Dimension};
use super::error;
use super::group::{Group, GroupMut};
use super::variable::{NcPutGet, Variable, VariableMut};
use crate::group::{get_parent_ncid_and_stem, try_get_ncid, try_get_parent_ncid_and_stem};

#[derive(Debug)]
#[repr(transparent)]
pub(crate) struct RawFile {
    ncid: nc_type,
}

impl Drop for RawFile {
    fn drop(&mut self) {
        unsafe {
            // Can't really do much with an error here
            let _err = error::checked(super::with_lock(|| nc_close(self.ncid)));
        }
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
            error::checked(super::with_lock(|| {
                nc_open(f.as_ptr().cast(), options.bits(), &mut ncid)
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
            error::checked(super::with_lock(|| {
                nc_create(f.as_ptr().cast(), options.bits(), &mut ncid)
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
            error::checked(super::with_lock(|| {
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
                error::checked(super::with_lock(|| {
                    nc_inq_path(self.0.ncid, &mut pathlen, std::ptr::null_mut())
                }))?;
            }
            let mut name = vec![0_u8; pathlen + 1];
            unsafe {
                error::checked(super::with_lock(|| {
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
        unsafe { error::checked(super::with_lock(|| nc_inq_format(self.ncid(), &mut format))) }
            .unwrap();

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
    pub fn types(&self) -> error::Result<impl Iterator<Item = super::types::VariableType>> {
        super::types::all_at_location(self.ncid()).map(|x| x.map(Result::unwrap))
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

    /// Create a variable in the dataset, with no data written into it
    ///
    /// # Examples
    /// ## Adding variables with different types
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut file = netcdf::create("file.nc")?;
    /// file.add_dimension("x", 10)?;
    /// file.add_dimension("y", 11)?;
    /// file.add_variable::<u8, _>("var_u8", &["y", "x"])?;
    /// file.add_variable::<i8, _>("var_i8", &["y", "x"])?;
    /// file.add_variable::<u64, _>("var_u64", &["y", "x"])?;
    /// file.add_variable::<f64, _>("var_f64", &["y", "x"])?;
    /// # Ok(())}
    /// ```
    /// ## Adding a scalar variable
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut file = netcdf::create("scalar.nc")?;
    /// file.add_variable::<u8, _>("var2", ())?;
    /// # Ok(())}
    /// ```
    /// ## Using dimension identifiers in place of names
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut file = netcdf::create("dimids.nc")?;
    /// let dimx = file.dimension("x").unwrap().identifier();
    /// let dimy = file.dimension("y").unwrap().identifier();
    /// file.add_variable::<u8, _>("var2", &[dimy, dimx])?;
    /// # Ok(())}
    ///```
    ///
    /// See [`AsNcDimensions`] for how to specify dimensions.
    pub fn add_variable<'f, T, D>(
        &'f mut self,
        name: &str,
        dims: D,
    ) -> error::Result<VariableMut<'f>>
    where
        T: NcPutGet,
        D: AsNcDimensions,
    {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.ncid(), name)?;
        VariableMut::add_from_dimids(ncid, T::NCTYPE, name, dims.get_dimensions(ncid)?)
    }

    /// Create a variable with the specified type
    pub fn add_variable_with_type<'f>(
        &'f mut self,
        name: &str,
        dims: &[&str],
        typ: &super::types::VariableType,
    ) -> error::Result<VariableMut<'f>> {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.ncid(), name)?;
        VariableMut::add_from_str(ncid, typ.id(), name, dims)
    }

    /// Add an opaque datatype, with `size` bytes
    pub fn add_opaque_type(
        &mut self,
        name: &str,
        size: usize,
    ) -> error::Result<super::types::OpaqueType> {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.ncid(), name)?;
        super::types::OpaqueType::add(ncid, name, size)
    }
    /// Add a variable length datatype
    pub fn add_vlen_type<T: NcPutGet>(
        &mut self,
        name: &str,
    ) -> error::Result<super::types::VlenType> {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.ncid(), name)?;
        super::types::VlenType::add::<T>(ncid, name)
    }
    /// Add an enum datatype
    pub fn add_enum_type<T: NcPutGet>(
        &mut self,
        name: &str,
        mappings: &[(&str, T)],
    ) -> error::Result<super::types::EnumType> {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.ncid(), name)?;
        super::types::EnumType::add::<T>(ncid, name, mappings)
    }

    /// Build a compound type
    pub fn add_compound_type(
        &mut self,
        name: &str,
    ) -> error::Result<super::types::CompoundBuilder> {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.ncid(), name)?;
        super::types::CompoundType::add(ncid, name)
    }

    /// Adds a variable with a basic type of string
    pub fn add_string_variable<'f>(
        &'f mut self,
        name: &str,
        dims: &[&str],
    ) -> error::Result<VariableMut<'f>> {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.ncid(), name)?;
        VariableMut::add_from_str(ncid, NC_STRING, name, dims)
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
