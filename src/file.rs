//! Open, create, and append netcdf files

#![allow(clippy::similar_names)]
use super::attribute::{AttrValue, Attribute};
use super::dimension::{self, Dimension};
use super::error;
use super::group::{Group, GroupMut};
use super::variable::{Numeric, Variable, VariableMut};
use super::LOCK;
use netcdf_sys::*;
use std::ffi::CString;
use std::marker::PhantomData;
use std::path;

#[derive(Debug)]
pub(crate) struct File {
    ncid: nc_type,
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            let _g = LOCK.lock().unwrap();
            // Can't really do much with an error here
            let _err = error::checked(nc_close(self.ncid));
        }
    }
}

impl File {
    /// Open a netCDF file in read only mode.
    ///
    /// Consider using [`netcdf::open`] instead to open with
    /// a generic `Path` object, and ensure read-only on
    /// the `File`
    pub(crate) fn open(path: &path::Path) -> error::Result<ReadOnlyFile> {
        let f = CString::new(path.to_str().unwrap()).unwrap();
        let mut ncid: nc_type = 0;
        unsafe {
            let _l = LOCK.lock().unwrap();
            error::checked(nc_open(f.as_ptr(), NC_NOWRITE, &mut ncid))?;
        }
        Ok(ReadOnlyFile(Self { ncid }))
    }

    #[allow(clippy::doc_markdown)]
    /// Open a netCDF file in append mode (read/write).
    /// The file must already exist.
    pub(crate) fn append(path: &path::Path) -> error::Result<MutableFile> {
        let f = CString::new(path.to_str().unwrap()).unwrap();
        let mut ncid: nc_type = -1;
        unsafe {
            let _g = LOCK.lock().unwrap();
            error::checked(nc_open(f.as_ptr(), NC_WRITE, &mut ncid))?;
        }

        Ok(MutableFile(ReadOnlyFile(Self { ncid })))
    }
    #[allow(clippy::doc_markdown)]
    /// Open a netCDF file in creation mode.
    ///
    /// Will overwrite existing file if any
    pub(crate) fn create(path: &path::Path) -> error::Result<MutableFile> {
        let f = CString::new(path.to_str().unwrap()).unwrap();
        let mut ncid: nc_type = -1;
        unsafe {
            let _g = LOCK.lock().unwrap();
            error::checked(nc_create(f.as_ptr(), NC_NETCDF4 | NC_CLOBBER, &mut ncid))?;
        }

        Ok(MutableFile(ReadOnlyFile(Self { ncid })))
    }

    #[cfg(feature = "memory")]
    pub(crate) fn open_from_memory<'buffer>(
        name: Option<&str>,
        mem: &'buffer [u8],
    ) -> error::Result<MemFile<'buffer>> {
        let cstr = std::ffi::CString::new(name.unwrap_or("/")).unwrap();
        let mut ncid = 0;
        unsafe {
            let _l = LOCK.lock().unwrap();
            error::checked(nc_open_mem(
                cstr.as_ptr(),
                NC_NOWRITE,
                mem.len(),
                mem.as_ptr() as *const u8 as *mut _,
                &mut ncid,
            ))?;
        }

        Ok(MemFile(ReadOnlyFile(Self { ncid }), PhantomData))
    }
}

#[derive(Debug)]
pub struct ReadOnlyFile(File);

impl ReadOnlyFile {
    /// path used to open/create the file
    ///
    /// #Errors
    ///
    /// Netcdf layer could fail, or the resulting path
    /// could contain an invalid UTF8 sequence
    pub fn path(&self) -> error::Result<String> {
        let name = {
            let mut pathlen = 0;
            unsafe {
                error::checked(nc_inq_path(self.0.ncid, &mut pathlen, std::ptr::null_mut()))?;
            }
            let mut name = vec![0_u8; pathlen as _];
            unsafe {
                error::checked(nc_inq_path(
                    self.0.ncid,
                    std::ptr::null_mut(),
                    name.as_mut_ptr() as *mut _,
                ))?;
            }
            name
        };

        Ok(String::from_utf8(name)?)
    }

    /// Main entrypoint for interacting with the netcdf file.
    pub fn root<'f>(&'f self) -> Group<'f> {
        Group {
            ncid: self.ncid(),
            _file: PhantomData,
        }
    }

    fn ncid(&self) -> nc_type {
        self.0.ncid
    }

    /// Get a variable from the group
    pub fn variable<'f>(&'f self, name: &str) -> error::Result<Option<Variable<'f>>> {
        Variable::find_from_name(self.ncid(), name)
    }
    /// Iterate over all variables in a group
    pub fn variables<'f>(
        &'f self,
    ) -> error::Result<impl Iterator<Item = error::Result<Variable<'f>>>> {
        super::variable::variables_at_ncid(self.ncid())
    }

    /// Get a single attribute
    pub fn attribute<'f>(&'f self, name: &str) -> error::Result<Option<Attribute<'f>>> {
        let _l = super::LOCK.lock().unwrap();
        Attribute::find_from_name(self.ncid(), None, name)
    }
    /// Get all attributes in the root group
    pub fn attributes<'f>(
        &'f self,
    ) -> error::Result<impl Iterator<Item = error::Result<Attribute<'f>>>> {
        let _l = super::LOCK.lock().unwrap();
        crate::attribute::AttributeIterator::new(self.0.ncid, None)
    }

    /// Get a single dimension
    pub fn dimension<'f>(&self, name: &str) -> error::Result<Option<Dimension<'f>>> {
        super::dimension::dimension_from_name(self.ncid(), name)
    }
    /// Iterator over all dimensions in the root group
    pub fn dimensions<'f>(
        &'f self,
    ) -> error::Result<impl Iterator<Item = error::Result<Dimension<'f>>>> {
        super::dimension::dimensions_from_location(self.ncid())
    }

    /// Get a group
    pub fn group<'f>(&'f self, name: &str) -> error::Result<Option<Group<'f>>> {
        super::group::group_from_name(self.ncid(), name)
    }
    /// Iterator over all subgroups in the root group
    pub fn groups<'f>(&'f self) -> error::Result<impl Iterator<Item = Group<'f>>> {
        super::group::groups_at_ncid(self.ncid())
    }
}

/// Mutable access to file
#[derive(Debug)]
pub struct MutableFile(ReadOnlyFile);

impl std::ops::Deref for MutableFile {
    type Target = ReadOnlyFile;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MutableFile {
    /// Mutable access to the root group
    pub fn root_mut<'f>(&'f mut self) -> GroupMut<'f> {
        GroupMut(self.root(), PhantomData)
    }

    /// Get a mutable variable from the group
    pub fn variable_mut<'f>(&'f mut self, name: &str) -> error::Result<Option<VariableMut<'f>>> {
        self.variable(name)
            .map(|var| var.map(|var| VariableMut(var, PhantomData)))
    }
    /// Iterate over all variables in the root group, with mutable access
    ///
    /// # Examples
    /// Use this to get multiple writable variables
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut file = netcdf::append("file.nc")?;
    /// let mut vars = file.variables_mut()?.collect::<Result<Vec<_>, _>>()?;
    /// vars[0].put_value(1_u8, Some(&[2, 5]))?;
    /// vars[1].put_value(1_u8, Some(&[5, 2]))?;
    /// # Ok(()) }
    /// ```
    pub fn variables_mut<'f>(
        &'f mut self,
    ) -> error::Result<impl Iterator<Item = error::Result<VariableMut<'f>>>> {
        self.variables()
            .map(|v| v.map(|var| var.map(|var| VariableMut(var, PhantomData))))
    }

    /// Mutable access to subgroup
    pub fn group_mut<'f>(&'f mut self, name: &str) -> error::Result<Option<GroupMut<'f>>> {
        self.group(name)
            .map(|g| g.map(|g| GroupMut(g, PhantomData)))
    }
    /// Iterator over all groups (mutable access)
    pub fn groups_mut<'f>(&'f mut self) -> error::Result<impl Iterator<Item = GroupMut<'f>>> {
        self.groups().map(|g| g.map(|g| GroupMut(g, PhantomData)))
    }

    /// Add an attribute to the root group
    pub fn add_attribute<'a, T>(&'a mut self, name: &str, val: T) -> error::Result<Attribute<'a>>
    where
        T: Into<AttrValue>,
    {
        let _l = LOCK.lock().unwrap();
        Attribute::put(self.ncid(), NC_GLOBAL, name, val.into())
    }

    /// Adds a dimension with the given name and size. A size of zero gives an unlimited dimension
    pub fn add_dimension<'f>(&'f mut self, name: &str, len: usize) -> error::Result<Dimension<'f>> {
        let _l = LOCK.lock().unwrap();
        super::dimension::add_dimension_at(self.ncid(), name, len)
    }
    /// Adds a dimension with unbounded size
    pub fn add_unlimited_dimension(&mut self, name: &str) -> error::Result<Dimension> {
        self.add_dimension(name, 0)
    }

    /// Add an empty group to the dataset
    pub fn add_group<'f>(&'f mut self, name: &str) -> error::Result<GroupMut<'f>> {
        let _l = LOCK.lock().unwrap();
        GroupMut::add_group_at(self.ncid(), name)
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
        T: Numeric,
    {
        let _l = LOCK.lock().unwrap();
        VariableMut::add_from_str(self.ncid(), T::NCTYPE, name, dims)
    }
    /// Adds a variable with a basic type of string
    pub fn add_string_variable<'f>(
        &'f mut self,
        name: &str,
        dims: &[&str],
    ) -> error::Result<VariableMut<'f>> {
        let _l = LOCK.lock().unwrap();
        VariableMut::add_from_str(self.ncid(), NC_STRING, name, dims)
    }
    /// Adds a variable from a set of unique identifiers, recursing upwards
    /// from the current group if necessary.
    pub fn add_variable_from_identifiers<'f, T>(
        &'f mut self,
        name: &str,
        dims: &[dimension::Identifier],
    ) -> error::Result<VariableMut<'f>>
    where
        T: Numeric,
    {
        let _l = LOCK.lock().unwrap();
        super::variable::add_variable_from_identifiers(self.ncid(), name, dims, T::NCTYPE)
    }
}

#[cfg(feature = "memory")]
/// The memory mapped file is kept in this structure to keep the
/// lifetime of the buffer longer than the file.
///
/// Access a [`ReadOnlyFile`] through the `Deref` trait,
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let buffer = &[0, 42, 1, 2];
/// let file = &netcdf::open_mem(None, buffer)?;
///
/// let variables = file.variables()?;
/// # Ok(()) }
/// ```
#[allow(clippy::module_name_repetitions)]
pub struct MemFile<'buffer>(ReadOnlyFile, std::marker::PhantomData<&'buffer [u8]>);

#[cfg(feature = "memory")]
impl<'a> std::ops::Deref for MemFile<'a> {
    type Target = ReadOnlyFile;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
