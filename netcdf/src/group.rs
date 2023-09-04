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
        Variable::find_from_path(self.id(), name.split('/')).unwrap()
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
        Attribute::find_from_path(self.ncid, None, name.split('/')).unwrap()
    }
    /// Get all attributes in the group
    pub fn attributes(&self) -> impl Iterator<Item = Attribute> {
        // Need to lock when reading the first attribute (per group)
        crate::attribute::AttributeIterator::new(self.ncid, None)
            .unwrap()
            .map(Result::unwrap)
    }

    /// Get a single dimension
    pub fn dimension<'g>(&'g self, name: &str) -> Option<Dimension<'g>>
    where
        'f: 'g,
    {
        super::dimension::dimension_from_name(self.id(), name).unwrap()
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
        Some(Group {
            ncid: group_from_path(self.id(), name.split('/')).unwrap()?,
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
        let (ncid, name) = super::group::get_subgroup_ncid_and_stem_from_path(self.id(), name)?;
        super::types::OpaqueType::add(ncid, name, size)
    }

    /// Add a variable length datatype
    pub fn add_vlen_type<T: NcPutGet>(
        &'f mut self,
        name: &str,
    ) -> error::Result<super::types::VlenType> {
        let (ncid, name) = super::group::get_subgroup_ncid_and_stem_from_path(self.id(), name)?;
        super::types::VlenType::add::<T>(ncid, name)
    }

    /// Add an enum datatype
    pub fn add_enum_type<T: NcPutGet>(
        &'f mut self,
        name: &str,
        mappings: &[(&str, T)],
    ) -> error::Result<super::types::EnumType> {
        let (ncid, name) = super::group::get_subgroup_ncid_and_stem_from_path(self.id(), name)?;
        super::types::EnumType::add::<T>(ncid, name, mappings)
    }

    /// Build a compound type
    pub fn add_compound_type(
        &mut self,
        name: &str,
    ) -> error::Result<super::types::CompoundBuilder> {
        let (ncid, name) = super::group::get_subgroup_ncid_and_stem_from_path(self.id(), name)?;
        super::types::CompoundType::add(ncid, name)
    }

    /// Add an attribute to the group
    pub fn add_attribute<'a, T>(&'a mut self, name: &str, val: T) -> error::Result<Attribute<'a>>
    where
        T: Into<AttrValue>,
    {
        let (ncid, name) = super::group::get_subgroup_ncid_and_stem_from_path(self.id(), name)?;
        Attribute::put(ncid, NC_GLOBAL, name, val.into())
    }

    /// Adds a dimension with the given name and size. A size of zero gives an unlimited dimension
    pub fn add_dimension<'g>(&'g mut self, name: &str, len: usize) -> error::Result<Dimension<'g>> {
        let (ncid, name) = super::group::get_subgroup_ncid_and_stem_from_path(self.id(), name)?;
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
                ncid: add_group_at_path(self.id(), name.split('/'))?,
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
        let (ncid, name) = super::group::get_subgroup_ncid_and_stem_from_path(self.id(), name)?;
        VariableMut::add_from_str(ncid, T::NCTYPE, name, dims)
    }
    /// Adds a variable with a basic type of string
    pub fn add_string_variable<'g>(
        &mut self,
        name: &str,
        dims: &[&str],
    ) -> error::Result<VariableMut<'g>> {
        let (ncid, name) = super::group::get_subgroup_ncid_and_stem_from_path(self.id(), name)?;
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
        let (ncid, name) = super::group::get_subgroup_ncid_and_stem_from_path(self.id(), name)?;
        super::variable::add_variable_from_identifiers(ncid, name, dims, T::NCTYPE)
    }

    /// Create a variable with the specified type
    pub fn add_variable_with_type(
        &'f mut self,
        name: &str,
        dims: &[&str],
        typ: &super::types::VariableType,
    ) -> error::Result<VariableMut<'f>> {
        let (ncid, name) = super::group::get_subgroup_ncid_and_stem_from_path(self.id(), name)?;
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

pub(crate) fn group_from_path<'i>(
    ncid: nc_type,
    path: impl Iterator<Item = &'i str>,
) -> error::Result<Option<nc_type>> {
    let mut e;
    let mut grpid = ncid;
    for name in path {
        let byte_name = super::utils::short_name_to_bytes(name)?;
        e = unsafe {
            super::with_lock(|| nc_inq_grp_ncid(grpid, byte_name.as_ptr().cast(), &mut grpid))
        };
        if e == NC_ENOGRP {
            return Ok(None);
        }
        error::checked(e)?;
    }
    Ok(Some(grpid))
}

pub(crate) fn add_group_at_path<'i>(
    ncid: nc_type,
    mut path: impl Iterator<Item = &'i str> + DoubleEndedIterator + Clone,
) -> error::Result<nc_type> {
    let name = path.next_back().unwrap();
    let ncid = match group_from_path(ncid, path.clone())? {
        Some(ncid) => ncid,
        None => add_group_at_path(ncid, path)?,
    };
    let byte_name = super::utils::short_name_to_bytes(name)?;
    let mut grpid = 0;
    unsafe {
        error::checked(super::with_lock(|| {
            nc_def_grp(ncid, byte_name.as_ptr().cast(), &mut grpid)
        }))?;
    }
    Ok(grpid)
}

pub(crate) fn get_subgroup_ncid_and_stem_from_path(
    ncid: nc_type,
    path: &str,
) -> error::Result<(nc_type, &str)> {
    let mut path = path.split('/');
    let name = path.next_back().unwrap();
    let ncid = super::group::group_from_path(ncid, path)?
        .ok_or(error::Error::Str("Missing child group".into()))?;
    Ok((ncid, name))
}
