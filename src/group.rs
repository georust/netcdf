//! All netcdf items belong in the root group, which can
//! be interacted with to get the underlying data

use super::attribute::AttrValue;
use super::attribute::Attribute;
use super::dimension::Dimension;
use super::error;
use super::variable::{Numeric, Variable, VariableMut};
use netcdf_sys::*;
use std::marker::PhantomData;

/// Main component of the netcdf format. Holds all variables,
/// attributes, and dimensions. A group can always see the parents items,
/// but a parent can not access a childs items.
#[derive(Debug, Clone)]
pub struct Group<'f> {
    pub(crate) ncid: nc_type,
    pub(crate) grpid: Option<nc_type>,
    pub(crate) _file: PhantomData<&'f nc_type>,
}

#[derive(Debug)]
/// Mutable access to a group
pub struct GroupMut<'f>(
    pub(crate) Group<'f>,
    pub(crate) PhantomData<&'f mut nc_type>,
);

impl<'f> std::ops::Deref for GroupMut<'f> {
    type Target = Group<'f>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'f> Group<'f> {
    /// Name of the current group
    pub fn name(&self) -> error::Result<String> {
        let mut name = vec![0_u8; NC_MAX_NAME as usize + 1];
        unsafe {
            error::checked(nc_inq_grpname(
                self.grpid.unwrap_or(self.ncid),
                name.as_mut_ptr() as *mut _,
            ))?;
        }
        let zeropos = name
            .iter()
            .position(|&x| x == 0)
            .unwrap_or_else(|| name.len());
        name.resize(zeropos, 0);

        Ok(String::from_utf8(name)?)
    }
    /// Get a variable from the group
    pub fn variable<'g>(&'g self, name: &str) -> Option<Variable<'f, 'g>> {
        todo!()
    }
    /// Iterate over all variables in a group
    pub fn variables<'g>(&'g self) -> impl Iterator<Item = Variable<'f, 'g>> {
        (0..).into_iter().map(|_| todo!())
    }
    /// Get a single attribute
    pub fn attribute<'a>(&'a self, name: &str) -> error::Result<Option<Attribute<'a>>> {
        let _l = super::LOCK.lock().unwrap();
        Attribute::find_from_name(self.ncid, None, name)
    }
    /// Get all attributes in the group
    pub fn attributes(&self) -> error::Result<impl Iterator<Item = error::Result<Attribute>>> {
        // Need to lock when reading the first attribute (per group)
        let _l = super::LOCK.lock().unwrap();
        crate::attribute::AttributeIterator::new(self.grpid.unwrap_or(self.ncid), None)
    }
    /// Get a single dimension
    pub fn dimension(&self, name: &str) -> Option<Dimension> {
        todo!()
    }
    /// Iterator over all dimensions
    pub fn dimensions<'g>(&'g self) -> impl Iterator<Item = Dimension<'g>> {
        (0..).into_iter().map(|_| todo!())
    }
    /// Get a group
    pub fn group(&self, name: &str) -> Option<Self> {
        let cname = std::ffi::CString::new(name).unwrap();
        let mut grpid = 0;
        unsafe {
            error::checked(nc_inq_grp_ncid(self.ncid, cname.as_ptr(), &mut grpid)).unwrap();
        }

        Some(Group {
            ncid: self.ncid,
            grpid: Some(grpid),
            _file: PhantomData,
        })
    }
    /// Iterator over all subgroups in this group
    pub fn groups<'g>(&'g self) -> impl Iterator<Item = Group<'g>> {
        (0..).into_iter().map(|_| todo!())
    }
}

impl<'f> GroupMut<'f> {
    /// Get a mutable variable from the group
    pub fn variable_mut<'g>(&'g mut self, name: &str) -> Option<VariableMut<'f, 'g>> {
        todo!()
    }
    /// Iterate over all variables in a group, with mutable access
    pub fn variables_mut<'g>(
        &'g mut self,
    ) -> error::Result<impl Iterator<Item = VariableMut<'f, 'g>>> {
        Ok((0..10).into_iter().map(|_| todo!()))
    }
    /// Mutable access to group
    pub fn group_mut(&'f mut self, name: &str) -> Option<Self> {
        todo!()
    }
    /// Iterator over all groups (mutable access)
    pub fn groups_mut(&'f mut self) -> error::Result<impl Iterator<Item = GroupMut<'f>>> {
        Ok((0..10).into_iter().map(|_| todo!()))
    }

    /// Add an attribute to the group
    pub fn add_attribute<'a, T>(&'a mut self, name: &str, val: T) -> error::Result<Attribute<'a>>
    where
        T: Into<AttrValue>,
    {
        Attribute::put(self.grpid.unwrap_or(self.ncid), NC_GLOBAL, name, val.into())
    }

    /// Adds a dimension with the given name and size. A size of zero gives an unlimited dimension
    pub fn add_dimension<'g>(&'g mut self, name: &str, len: usize) -> error::Result<Dimension<'g>> {
        use std::ffi::CString;

        let mut dimid = 0;
        let cname = CString::new(name).unwrap();

        unsafe {
            error::checked(nc_def_dim(
                self.grpid.unwrap_or(self.ncid),
                cname.as_ptr(),
                len,
                &mut dimid,
            ))?;
        }

        Ok(Dimension {
            len: core::num::NonZeroUsize::new(len),
            id: super::dimension::Identifier {
                ncid: self.grpid.unwrap_or(self.ncid),
                dimid,
            },
            _group: PhantomData,
        })
    }

    /// Adds a dimension with unbounded size
    pub fn add_unlimited_dimension(&mut self, name: &str) -> error::Result<Dimension> {
        self.add_dimension(name, 0)
    }

    /// Add an empty group to the dataset
    pub fn add_group(&mut self, name: &str) -> error::Result<Self> {
        if self.group(name).is_some() {
            return Err(error::Error::AlreadyExists(name.to_string()));
        }
        let cstr = std::ffi::CString::new(name).unwrap();
        let mut grpid = 0;
        unsafe {
            error::checked(nc_def_grp(
                self.grpid.unwrap_or(self.ncid),
                cstr.as_ptr(),
                &mut grpid,
            ))?;
        }

        Ok(Self(
            Group {
                ncid: self.grpid.unwrap_or(self.ncid),
                grpid: Some(grpid),
                _file: PhantomData,
            },
            PhantomData,
        ))
    }
    /// Adds a variable from a set of unique identifiers, recursing upwards
    /// from the current group if necessary.
    pub fn add_variable_from_identifiers<T>(
        &mut self,
        name: &str,
        dims: &[super::dimension::Identifier],
    ) -> error::Result<VariableMut>
    where
        T: Numeric,
    {
        todo!()
    }

    /// Create a Variable into the dataset, with no data written into it
    ///
    /// Dimensions are identified using the name of the dimension, and will recurse upwards
    /// if not found in the current group.
    pub fn add_variable<'g, T>(
        &'g mut self,
        name: &str,
        dims: &[&str],
    ) -> error::Result<VariableMut<'f, 'g>>
    where
        T: Numeric,
    {
        VariableMut::add_from_str(self.id(), T::NCTYPE, name, dims)
    }

    fn id(&self) -> nc_type {
        self.grpid.unwrap_or(self.ncid)
    }

    /// Adds a variable with a basic type of string
    pub fn add_string_variable(&mut self, name: &str, dims: &[&str]) -> error::Result<VariableMut> {
        VariableMut::add_from_str(self.id(), NC_STRING, name, dims)
    }
}

impl<'f> Group<'f> {
    /// Asserts all dimensions exists, and gets a copy of these
    /// (will be moved into a Variable)
    fn find_dimensions(&self, dims: &[&str]) -> error::Result<Vec<Dimension>> {
        todo!()
    }

    pub(crate) fn parent(&self) -> Option<Self> {
        unimplemented!()
    }
}
