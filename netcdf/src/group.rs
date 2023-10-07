//! All netcdf items belong in the root group, which can
//! be interacted with to get the underlying data

use super::attribute::AttrValue;
use super::attribute::Attribute;
use super::dimension::Dimension;
use super::error;
use super::variable::{NcPutGet, Variable, VariableMut};
use netcdf_sys::*;
use std::convert::TryInto;
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
#[allow(clippy::module_name_repetitions)]
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
    pub fn name(&self) -> String {
        let mut name = vec![0_u8; NC_MAX_NAME as usize + 1];
        unsafe {
            error::checked(super::with_lock(|| {
                nc_inq_grpname(self.ncid, name.as_mut_ptr().cast())
            }))
            .unwrap();
        }
        let zeropos = name.iter().position(|&x| x == 0).unwrap_or(name.len());
        name.resize(zeropos, 0);

        String::from_utf8(name).expect("Group did not have a valid name")
    }
    /// Internal ncid of the group
    fn id(&self) -> nc_type {
        self.ncid
    }

    /// Get a variable from the group
    pub fn variable<'g>(&'g self, name: &str) -> Option<Variable<'g>>
    where
        'f: 'g,
    {
        let (ncid, name) = super::group::try_get_parent_ncid_and_stem(self.id(), name).unwrap()?;
        Variable::find_from_name(ncid, name).unwrap()
    }
    /// Iterate over all variables in a group
    pub fn variables<'g>(&'g self) -> impl Iterator<Item = Variable<'g>>
    where
        'f: 'g,
    {
        super::variable::variables_at_ncid(self.id())
            .unwrap()
            .map(Result::unwrap)
    }

    /// Get a single attribute
    pub fn attribute<'a>(&'a self, name: &str) -> Option<Attribute<'a>> {
        let (ncid, name) = try_get_parent_ncid_and_stem(self.id(), name).unwrap()?;
        Attribute::find_from_name(ncid, None, name).unwrap()
    }
    /// Get all attributes in the group
    pub fn attributes(&self) -> impl Iterator<Item = Attribute> {
        // Need to lock when reading the first attribute (per group)
        crate::attribute::AttributeIterator::new(self.ncid, None)
            .unwrap()
            .map(Result::unwrap)
    }
    /// Get the attribute value
    pub fn attribute_value(&self, name: &str) -> Option<error::Result<AttrValue>> {
        self.attribute(name).as_ref().map(Attribute::value)
    }

    /// Get a single dimension
    pub fn dimension<'g>(&'g self, name: &str) -> Option<Dimension<'g>>
    where
        'f: 'g,
    {
        let (ncid, name) = super::group::try_get_parent_ncid_and_stem(self.id(), name).unwrap()?;
        super::dimension::dimension_from_name(ncid, name).unwrap()
    }
    /// Iterator over all dimensions
    pub fn dimensions<'g>(&'g self) -> impl Iterator<Item = Dimension<'g>>
    where
        'f: 'g,
    {
        super::dimension::dimensions_from_location(self.id())
            .unwrap()
            .map(Result::unwrap)
    }

    /// Get a group
    pub fn group<'g>(&'g self, name: &str) -> Option<Group<'g>>
    where
        'f: 'g,
    {
        // We are in a group, must support netCDF-4
        let (ncid, name) = get_parent_ncid_and_stem(self.id(), name).unwrap();
        Some(Group {
            ncid: try_get_ncid(ncid, name).unwrap()?,
            _file: PhantomData,
        })
    }
    /// Iterator over all subgroups in this group
    pub fn groups<'g>(&'g self) -> impl Iterator<Item = Group<'g>>
    where
        'f: 'g,
    {
        groups_at_ncid(self.id()).unwrap()
    }

    /// Return all types in this group
    pub fn types(&self) -> impl Iterator<Item = super::types::VariableType> {
        super::types::all_at_location(self.ncid)
            .map(|x| x.map(Result::unwrap))
            .unwrap()
    }
}

impl<'f> GroupMut<'f> {
    /// Get a mutable variable from the group
    pub fn variable_mut<'g>(&'g mut self, name: &str) -> Option<VariableMut<'g>>
    where
        'f: 'g,
    {
        self.variable(name).map(|v| VariableMut(v, PhantomData))
    }
    /// Iterate over all variables in a group, with mutable access
    pub fn variables_mut<'g>(&'g mut self) -> impl Iterator<Item = VariableMut<'g>>
    where
        'f: 'g,
    {
        self.variables().map(|var| VariableMut(var, PhantomData))
    }

    /// Mutable access to subgroup
    pub fn group_mut<'g>(&'g mut self, name: &str) -> Option<GroupMut<'g>>
    where
        'f: 'g,
    {
        self.group(name).map(|g| GroupMut(g, PhantomData))
    }
    /// Iterator over all groups (mutable access)
    pub fn groups_mut<'g>(&'g mut self) -> impl Iterator<Item = GroupMut<'g>>
    where
        'f: 'g,
    {
        self.groups().map(|g| GroupMut(g, PhantomData))
    }

    /// Add an opaque datatype, with `size` bytes
    pub fn add_opaque_type(
        &'f mut self,
        name: &str,
        size: usize,
    ) -> error::Result<super::types::OpaqueType> {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.id(), name)?;
        super::types::OpaqueType::add(ncid, name, size)
    }

    /// Add a variable length datatype
    pub fn add_vlen_type<T: NcPutGet>(
        &'f mut self,
        name: &str,
    ) -> error::Result<super::types::VlenType> {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.id(), name)?;
        super::types::VlenType::add::<T>(ncid, name)
    }

    /// Add an enum datatype
    pub fn add_enum_type<T: NcPutGet>(
        &'f mut self,
        name: &str,
        mappings: &[(&str, T)],
    ) -> error::Result<super::types::EnumType> {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.id(), name)?;
        super::types::EnumType::add::<T>(ncid, name, mappings)
    }

    /// Build a compound type
    pub fn add_compound_type(
        &mut self,
        name: &str,
    ) -> error::Result<super::types::CompoundBuilder> {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.id(), name)?;
        super::types::CompoundType::add(ncid, name)
    }

    /// Add an attribute to the group
    pub fn add_attribute<'a, T>(&'a mut self, name: &str, val: T) -> error::Result<Attribute<'a>>
    where
        T: Into<AttrValue>,
    {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.id(), name)?;
        Attribute::put(ncid, NC_GLOBAL, name, val.into())
    }

    /// Adds a dimension with the given name and size. A size of zero gives an unlimited dimension
    pub fn add_dimension<'g>(&'g mut self, name: &str, len: usize) -> error::Result<Dimension<'g>> {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.id(), name)?;
        super::dimension::add_dimension_at(ncid, name, len)
    }

    /// Adds a dimension with unbounded size
    pub fn add_unlimited_dimension<'g>(&'g mut self, name: &str) -> error::Result<Dimension<'g>> {
        self.add_dimension(name, 0)
    }

    /// Add an empty group to the dataset
    pub fn add_group<'g>(&'g mut self, name: &str) -> error::Result<GroupMut<'g>>
    where
        'f: 'g,
    {
        Ok(Self(
            Group {
                ncid: add_group_at_path(self.id(), name)?,
                _file: PhantomData,
            },
            PhantomData,
        ))
    }

    /// Create a Variable into the dataset, with no data written into it
    ///
    /// Dimensions are identified using the name of the dimension, and will recurse upwards
    /// if not found in the current group.
    pub fn add_variable<'g, T>(
        &'g mut self,
        name: &str,
        dims: &[&str],
    ) -> error::Result<VariableMut<'g>>
    where
        T: NcPutGet,
        'f: 'g,
    {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.id(), name)?;
        VariableMut::add_from_str(ncid, T::NCTYPE, name, dims)
    }
    /// Adds a variable with a basic type of string
    pub fn add_string_variable<'g>(
        &mut self,
        name: &str,
        dims: &[&str],
    ) -> error::Result<VariableMut<'g>> {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.id(), name)?;
        VariableMut::add_from_str(ncid, NC_STRING, name, dims)
    }
    /// Adds a variable from a set of unique identifiers, recursing upwards
    /// from the current group if necessary.
    pub fn add_variable_from_identifiers<'g, T>(
        &'g mut self,
        name: &str,
        dims: &[super::dimension::Identifier],
    ) -> error::Result<VariableMut<'g>>
    where
        T: NcPutGet,
    {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.id(), name)?;
        super::variable::add_variable_from_identifiers(ncid, name, dims, T::NCTYPE)
    }

    /// Create a variable with the specified type
    pub fn add_variable_with_type(
        &'f mut self,
        name: &str,
        dims: &[&str],
        typ: &super::types::VariableType,
    ) -> error::Result<VariableMut<'f>> {
        let (ncid, name) = super::group::get_parent_ncid_and_stem(self.id(), name)?;
        VariableMut::add_from_str(ncid, typ.id(), name, dims)
    }
}

pub(crate) fn groups_at_ncid<'f>(ncid: nc_type) -> error::Result<impl Iterator<Item = Group<'f>>> {
    let mut num_grps = 0;
    unsafe {
        error::checked(super::with_lock(|| {
            nc_inq_grps(ncid, &mut num_grps, std::ptr::null_mut())
        }))?;
    }
    let mut grps = vec![0; num_grps.try_into()?];
    unsafe {
        error::checked(super::with_lock(|| {
            nc_inq_grps(ncid, std::ptr::null_mut(), grps.as_mut_ptr())
        }))?;
    }
    Ok(grps.into_iter().map(|id| Group {
        ncid: id,
        _file: PhantomData,
    }))
}

pub(crate) fn add_group_at_path(mut ncid: nc_type, path: &str) -> error::Result<nc_type> {
    let mut path = path.split('/');
    let name = path.next_back().unwrap();
    for name in path {
        ncid = match try_get_ncid(ncid, name)? {
            Some(ncid) => ncid,
            None => add_group(ncid, name)?,
        }
    }
    add_group(ncid, name)
}

pub(crate) fn add_group(mut ncid: nc_type, name: &str) -> error::Result<nc_type> {
    let byte_name = super::utils::short_name_to_bytes(name)?;
    unsafe {
        error::checked(super::with_lock(|| {
            nc_def_grp(ncid, byte_name.as_ptr().cast(), &mut ncid)
        }))?;
    }
    Ok(ncid)
}

pub(crate) fn try_get_ncid(mut ncid: nc_type, name: &str) -> error::Result<Option<nc_type>> {
    let byte_name = super::utils::short_name_to_bytes(name)?;
    let e =
        unsafe { super::with_lock(|| nc_inq_grp_ncid(ncid, byte_name.as_ptr().cast(), &mut ncid)) };
    if e == NC_ENOGRP {
        return Ok(None);
    }
    error::checked(e)?;
    Ok(Some(ncid))
}

pub(crate) fn try_get_parent_ncid_and_stem(
    mut ncid: nc_type,
    path: &str,
) -> error::Result<Option<(nc_type, &str)>> {
    let mut path = path.split('/');
    let name = path.next_back().unwrap();
    for name in path {
        ncid = match try_get_ncid(ncid, name)? {
            None => return Ok(None),
            Some(ncid) => ncid,
        }
    }
    Ok(Some((ncid, name)))
}

pub(crate) fn get_parent_ncid_and_stem(
    ncid: nc_type,
    path: &str,
) -> error::Result<(nc_type, &str)> {
    try_get_parent_ncid_and_stem(ncid, path)?
        .ok_or_else(|| error::Error::Str("One of the child groups does not exist".into()))
}
