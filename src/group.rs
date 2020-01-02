//! All netcdf items belong in the root group, which can
//! be interacted with to get the underlying data

use super::attribute::AttrValue;
use super::attribute::Attribute;
use super::dimension::Dimension;
use super::error;
use super::variable::{Numeric, Variable};
use netcdf_sys::*;
use std::cell::UnsafeCell;
use std::rc::{Rc, Weak};

/// Main component of the netcdf format. Holds all variables,
/// attributes, and dimensions. A group can always see the parents items,
/// but a parent can not access a childs items.
#[derive(Debug)]
pub struct Group {
    pub(crate) name: String,
    pub(crate) ncid: nc_type,
    pub(crate) grpid: Option<nc_type>,
    pub(crate) variables: Vec<Variable>,
    pub(crate) dimensions: Vec<Dimension>,
    pub(crate) groups: Vec<Rc<UnsafeCell<Group>>>,
    /// Do not mutate parent, only for walking and getting dimensions
    /// and types. Use the `parents` iterator for walking upwards.
    ///
    /// Contains `None` only when `Group` is the root node
    pub(crate) parent: Option<Weak<UnsafeCell<Group>>>,
    /// Given as `parent` when supplying to child groups.
    ///
    /// Should never be `None` (this is just to be able
    /// to get a `Weak` into itself
    pub(crate) this: Option<Weak<UnsafeCell<Group>>>,
}

impl Group {
    /// Name of the current group
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Get a variable from the group
    pub fn variable(&self, name: &str) -> Option<&Variable> {
        self.variables().find(|x| x.name().unwrap() == name)
    }
    /// Iterate over all variables in a group
    pub fn variables(&self) -> impl Iterator<Item = &Variable> {
        self.variables.iter()
    }
    /// Get a mutable variable from the group
    pub fn variable_mut(&mut self, name: &str) -> Option<&mut Variable> {
        self.variables_mut().find(|x| x.name().unwrap() == name)
    }
    /// Iterate over all variables in a group, with mutable access
    pub fn variables_mut(&mut self) -> impl Iterator<Item = &mut Variable> {
        self.variables.iter_mut()
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
    pub fn dimension(&self, name: &str) -> Option<&Dimension> {
        self.dimensions().find(|x| x.name().unwrap() == name)
    }
    /// Iterator over all dimensions
    pub fn dimensions(&self) -> impl Iterator<Item = &Dimension> {
        // Need to lock when reading the first attribute (per group)
        self.dimensions.iter()
    }
    /// Get a group
    pub fn group(&self, name: &str) -> Option<&Self> {
        self.groups().find(|x| x.name() == name)
    }
    /// Iterator over all groups
    pub fn groups(&self) -> impl Iterator<Item = &Self> {
        self.groups.iter().map(|x| unsafe { &*x.get() })
    }
    /// Mutable access to group
    pub fn group_mut(&mut self, name: &str) -> Option<&mut Self> {
        self.groups_mut().find(|x| x.name() == name)
    }
    /// Iterator over all groups (mutable access)
    pub fn groups_mut(&mut self) -> impl Iterator<Item = &mut Self> {
        // Takes self as &mut
        self.groups.iter_mut().map(|x| unsafe { &mut *x.get() })
    }
}

impl Group {
    /// Add an attribute to the group
    pub fn add_attribute<'a, T>(&'a mut self, name: &str, val: T) -> error::Result<Attribute<'a>>
    where
        T: Into<AttrValue>,
    {
        Attribute::put(self.grpid.unwrap_or(self.ncid), NC_GLOBAL, name, val.into())
    }

    /// Adds a dimension with the given name and size. A size of zero gives an unlimited dimension
    pub fn add_dimension(&mut self, name: &str, len: usize) -> error::Result<&Dimension> {
        if self.dimension(name).is_some() {
            return Err(error::Error::AlreadyExists(format!("dimension {}", name)));
        }

        let d = Dimension::new(self.grpid.unwrap_or(self.ncid), name.to_string(), len)?;
        self.dimensions.push(d);

        Ok(self.dimension(name).unwrap())
    }

    /// Adds a dimension with unbounded size
    pub fn add_unlimited_dimension(&mut self, name: &str) -> error::Result<&Dimension> {
        self.add_dimension(name, 0)
    }

    /// Add an empty group to the dataset
    pub fn add_group(&mut self, name: &str) -> error::Result<&mut Self> {
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

        let g = Rc::new(UnsafeCell::new(Self {
            ncid: self.grpid.unwrap_or(self.ncid),
            name: name.to_string(),
            grpid: Some(grpid),
            dimensions: Vec::default(),
            groups: Vec::default(),
            variables: Vec::default(),
            parent: Some(self.this.clone().unwrap()),
            this: None,
        }));
        {
            let gref = Some(Rc::downgrade(&g));
            let g = unsafe { &mut *g.get() };
            g.this = gref;
        }
        self.groups.push(g);
        Ok(self.group_mut(name).unwrap())
    }

    /// Asserts all dimensions exists, and gets a copy of these
    /// (will be moved into a Variable)
    fn find_dimensions(&self, dims: &[&str]) -> error::Result<Vec<Dimension>> {
        let (d, e): (Vec<_>, Vec<_>) = dims
            .iter()
            .map(|name| {
                if let Some(x) = self.dimension(name) {
                    return Ok(x);
                }
                for pdim in self.parents() {
                    if let Some(x) = pdim.dimension(name) {
                        return Ok(x);
                    }
                }
                Err(*name)
            })
            .partition(Result::is_ok);

        if !e.is_empty() {
            let mut s = String::new();
            s.push_str("dimension(s)");
            for x in e {
                s.push(' ');
                s.push_str(x.unwrap_err());
            }
            return Err(error::Error::NotFound(s));
        }

        let d = d
            .into_iter()
            .map(Result::unwrap)
            .cloned()
            .collect::<Vec<_>>();
        Ok(d)
    }

    pub(crate) fn parents(&self) -> impl Iterator<Item = &Self> {
        ParentIterator::new(self)
    }

    /// Adds a variable from a set of unique identifiers, recursing upwards
    /// from the current group if necessary.
    pub fn add_variable_from_identifiers<T>(
        &mut self,
        name: &str,
        dims: &[super::dimension::Identifier],
    ) -> error::Result<&mut Variable>
    where
        T: Numeric,
    {
        if self.variable(name).is_some() {
            return Err(error::Error::AlreadyExists(format!("variable {}", name)));
        }
        let mut d: Vec<_> = Vec::default();
        for (i, dim) in dims.iter().enumerate() {
            let id = dim.identifier;
            let found_dim = match self
                .dimensions()
                .find(|&x| x.ncid == dim.ncid && x.id == id)
            {
                Some(x) => x.clone(),
                None => match self
                    .parents()
                    .flat_map(Self::dimensions)
                    .find(|d| d.ncid == dim.ncid && d.id == id)
                {
                    Some(d) => d.clone(),
                    None => return Err(error::Error::NotFound(format!("dimension #{}", i))),
                },
            };
            d.push(found_dim);
        }

        let var = Variable::new(self.grpid.unwrap_or(self.ncid), name, d, T::NCTYPE)?;
        self.variables.push(var);
        Ok(self.variable_mut(name).unwrap())
    }

    /// Create a Variable into the dataset, with no data written into it
    ///
    /// Dimensions are identified using the name of the dimension, and will recurse upwards
    /// if not found in the current group.
    pub fn add_variable<T>(&mut self, name: &str, dims: &[&str]) -> error::Result<&mut Variable>
    where
        T: Numeric,
    {
        if self.variable(name).is_some() {
            return Err(error::Error::AlreadyExists("variable".into()));
        }

        let d = self.find_dimensions(dims)?;
        let var = Variable::new(self.grpid.unwrap_or(self.ncid), name, d, T::NCTYPE)?;

        self.variables.push(var);
        Ok(self.variable_mut(name).unwrap())
    }

    /// Adds a variable with a basic type of string
    pub fn add_string_variable(
        &mut self,
        name: &str,
        dims: &[&str],
    ) -> error::Result<&mut Variable> {
        if self.variable(name).is_some() {
            return Err(error::Error::AlreadyExists("variable".into()));
        }

        let d = self.find_dimensions(dims)?;
        let var = Variable::new(self.grpid.unwrap_or(self.ncid), name, d, NC_STRING)?;

        self.variables.push(var);
        Ok(self.variable_mut(name).unwrap())
    }
}

struct ParentIterator<'a> {
    g: Weak<UnsafeCell<Group>>,
    _phantom: std::marker::PhantomData<&'a Group>,
}

impl<'a> ParentIterator<'a> {
    fn new(g: &Group) -> Self {
        Self {
            g: g.this.clone().unwrap(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> Iterator for ParentIterator<'a> {
    type Item = &'a Group;

    fn next(&mut self) -> Option<Self::Item> {
        let g: &Group = unsafe { &*self.g.upgrade().unwrap().get() };
        let p = match &g.parent {
            None => return None,
            Some(p) => p,
        };
        self.g = p.clone();
        Some(unsafe { &*self.g.upgrade().unwrap().get() })
    }
}
