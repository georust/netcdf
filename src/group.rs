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
            error::checked(nc_inq_grpname(self.ncid, name.as_mut_ptr() as *mut _))?;
        }
        let zeropos = name
            .iter()
            .position(|&x| x == 0)
            .unwrap_or_else(|| name.len());
        name.resize(zeropos, 0);

        Ok(String::from_utf8(name)?)
    }
    /// Internal ncid of the group
    fn id(&self) -> nc_type {
        self.ncid
    }

    /// Get a variable from the group
    pub fn variable<'g>(&'g self, name: &str) -> error::Result<Option<Variable<'f, 'g>>> {
        Variable::find_from_name(self.id(), name)
    }
    /// Iterate over all variables in a group
    pub fn variables<'g>(
        &'g self,
    ) -> error::Result<impl Iterator<Item = error::Result<Variable<'f, 'g>>>> {
        super::variable::variables_at_ncid(self.id())
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
        crate::attribute::AttributeIterator::new(self.ncid, None)
    }
    /// Get a single dimension
    pub fn dimension(&self, name: &str) -> error::Result<Option<Dimension>> {
        super::dimension::dimension_from_name(self.id(), name)
    }
    /// Iterator over all dimensions
    pub fn dimensions<'g>(
        &'g self,
    ) -> error::Result<impl Iterator<Item = error::Result<Dimension<'g>>>> {
        super::dimension::dimensions_from_location(self.id())
    }
    /// Get a group
    pub fn group(&self, name: &str) -> error::Result<Option<Self>> {
        group_from_name(self.id(), name)
    }
    /// Iterator over all subgroups in this group
    pub fn groups<'g>(&'g self) -> error::Result<impl Iterator<Item = Group<'g>>> {
        groups_at_ncid(self.id())
    }
}

impl<'f> GroupMut<'f> {
    /// Get a mutable variable from the group
    pub fn variable_mut<'g>(
        &'g mut self,
        name: &str,
    ) -> error::Result<Option<VariableMut<'f, 'g>>> {
        self.variable(name)
            .map(|v| v.map(|v| VariableMut(v, PhantomData)))
    }
    /// Iterate over all variables in a group, with mutable access
    pub fn variables_mut<'g>(
        &'g mut self,
    ) -> error::Result<impl Iterator<Item = error::Result<VariableMut<'f, 'g>>>> {
        self.variables()
            .map(|var| var.map(|var| var.map(|var| VariableMut(var, PhantomData))))
    }
    /// Mutable access to group
    pub fn group_mut(&'f mut self, name: &str) -> error::Result<Option<Self>> {
        self.group(name)
            .map(|g| g.map(|g| GroupMut(g, PhantomData)))
    }
    /// Iterator over all groups (mutable access)
    pub fn groups_mut(&'f mut self) -> error::Result<impl Iterator<Item = GroupMut<'f>>> {
        self.groups().map(|g| g.map(|g| GroupMut(g, PhantomData)))
    }

    /// Add an attribute to the group
    pub fn add_attribute<'a, T>(&'a mut self, name: &str, val: T) -> error::Result<Attribute<'a>>
    where
        T: Into<AttrValue>,
    {
        Attribute::put(self.ncid, NC_GLOBAL, name, val.into())
    }

    /// Adds a dimension with the given name and size. A size of zero gives an unlimited dimension
    pub fn add_dimension<'g>(&'g mut self, name: &str, len: usize) -> error::Result<Dimension<'g>> {
        super::dimension::add_dimension_at(self.id(), name, len)
    }

    /// Adds a dimension with unbounded size
    pub fn add_unlimited_dimension(&mut self, name: &str) -> error::Result<Dimension> {
        self.add_dimension(name, 0)
    }

    pub(crate) fn add_group_at(ncid: nc_type, name: &str) -> error::Result<Self> {
        let cstr = std::ffi::CString::new(name).unwrap();
        let mut grpid = 0;
        unsafe {
            error::checked(nc_def_grp(ncid, cstr.as_ptr(), &mut grpid))?;
        }

        Ok(Self(
            Group {
                ncid: grpid,
                _file: PhantomData,
            },
            PhantomData,
        ))
    }

    /// Add an empty group to the dataset
    pub fn add_group(&mut self, name: &str) -> error::Result<Self> {
        Self::add_group_at(self.id(), name)
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
        super::variable::add_variable_from_identifiers(self.id(), name, dims, T::NCTYPE)
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

    /// Adds a variable with a basic type of string
    pub fn add_string_variable(&mut self, name: &str, dims: &[&str]) -> error::Result<VariableMut> {
        VariableMut::add_from_str(self.id(), NC_STRING, name, dims)
    }
}

pub(crate) fn groups_at_ncid<'f>(ncid: nc_type) -> error::Result<impl Iterator<Item = Group<'f>>> {
    let mut ngrps = 0;
    unsafe {
        error::checked(nc_inq_grps(ncid, &mut ngrps, std::ptr::null_mut()))?;
    }
    let mut grps = vec![0; ngrps as _];
    unsafe {
        error::checked(nc_inq_grps(ncid, std::ptr::null_mut(), grps.as_mut_ptr()))?;
    }
    Ok(grps.into_iter().map(|id| Group {
        ncid: id,
        _file: PhantomData,
    }))
}

pub(crate) fn group_from_name<'f>(ncid: nc_type, name: &str) -> error::Result<Option<Group<'f>>> {
    let cname = std::ffi::CString::new(name).unwrap();
    let mut grpid = 0;
    let e = unsafe { nc_inq_grp_ncid(ncid, cname.as_ptr(), &mut grpid) };
    if e == NC_ENOTFOUND {
        return Ok(None);
    } else {
        error::checked(e)?;
    }
    Ok(Some(Group {
        ncid: grpid,
        _file: PhantomData,
    }))
}
