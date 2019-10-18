use super::attribute::AttrValue;
use super::attribute::Attribute;
use super::dimension::Dimension;
use super::error;
use super::variable::{Numeric, Variable};
use super::HashMap;
use netcdf_sys::*;

#[derive(Debug)]
pub struct Group {
    pub(crate) name: String,
    pub(crate) ncid: nc_type,
    pub(crate) grpid: Option<nc_type>,
    pub(crate) variables: HashMap<String, Variable>,
    pub(crate) attributes: HashMap<String, Attribute>,
    pub(crate) parent_dimensions: Vec<HashMap<String, Dimension>>,
    pub(crate) dimensions: HashMap<String, Dimension>,
    pub(crate) groups: HashMap<String, Group>,
}

impl Group {
    /// Name of the current group
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Get a variable from the group
    pub fn variable(&self, name: &str) -> Option<&Variable> {
        self.variables.get(name)
    }
    /// Iterate over all variables in a group
    pub fn variables(&self) -> impl Iterator<Item = &Variable> {
        self.variables.values()
    }
    /// Get a mutable variable from the group
    pub fn variable_mut(&mut self, name: &str) -> Option<&mut Variable> {
        self.variables.get_mut(name)
    }
    /// Iterate over all variables in a group, with mutable access
    pub fn variables_mut(&mut self) -> impl Iterator<Item = &mut Variable> {
        self.variables.values_mut()
    }
    /// Get a single attribute
    pub fn attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.get(name)
    }
    /// Get all attributes
    pub fn attributes(&self) -> impl Iterator<Item = &Attribute> {
        self.attributes.values()
    }
    /// Get a single dimension
    pub fn dimension(&self, name: &str) -> Option<&Dimension> {
        self.dimensions.get(name)
    }
    /// Iterator over all dimensions
    pub fn dimensions(&self) -> impl Iterator<Item = &Dimension> {
        self.dimensions.values()
    }
    /// Get a group
    pub fn group(&self, name: &str) -> Option<&Group> {
        self.groups.get(name)
    }
    /// Iterator over all groups
    pub fn groups(&self) -> impl Iterator<Item = &Group> {
        self.groups.values()
    }
    /// Mutable access to group
    pub fn group_mut(&mut self, name: &str) -> Option<&mut Group> {
        self.groups.get_mut(name)
    }
    /// Iterator over all groups (mutable access)
    pub fn groups_mut(&mut self) -> impl Iterator<Item = &mut Group> {
        self.groups.values_mut()
    }
}

impl Group {
    pub fn add_attribute<T>(&mut self, name: &str, val: T) -> error::Result<()>
    where
        T: Into<AttrValue>,
    {
        let att = Attribute::put(self.grpid.unwrap_or(self.ncid), NC_GLOBAL, name, val.into())?;
        self.attributes.insert(name.to_string(), att);
        Ok(())
    }

    /// Adds a dimension with the given name and size. A size of zero gives an unlimited dimension
    pub fn add_dimension(&mut self, name: &str, len: usize) -> error::Result<&mut Dimension> {
        if self.dimensions.contains_key(name) {
            return Err(error::Error::AlreadyExists("dimension".into()));
        }

        let d = Dimension::new(self.grpid.unwrap_or(self.ncid), name, len)?;
        self.dimensions.insert(name.into(), d.clone());

        fn recursively_add_dim(depth: usize, name: &str, d: &Dimension, g: &mut Group) {
            for (_, grp) in g.groups.iter_mut() {
                grp.parent_dimensions[depth].insert(name.to_string(), d.clone());
                recursively_add_dim(depth, name, d, grp);
            }
        }

        let mydepth = self.parent_dimensions.len();
        for (_, grp) in self.groups.iter_mut() {
            recursively_add_dim(mydepth, name, &d, grp);
        }

        Ok(self.dimensions.get_mut(name).unwrap())
    }

    /// Adds a dimension with unbounded size
    pub fn add_unlimited_dimension(&mut self, name: &str) -> error::Result<&mut Dimension> {
        self.add_dimension(name, 0)
    }

    /// Add an empty group to the dataset
    pub fn add_group(&mut self, name: &str) -> error::Result<&mut Group> {
        let cstr = std::ffi::CString::new(name).unwrap();
        let mut grpid = 0;
        unsafe {
            error::checked(nc_def_grp(
                self.grpid.unwrap_or(self.ncid),
                cstr.as_ptr(),
                &mut grpid,
            ))?;
        }

        let mut parent_dimensions = self.parent_dimensions.clone();
        parent_dimensions.push(self.dimensions.clone());

        let g = Group {
            ncid: self.grpid.unwrap_or(self.ncid),
            name: name.to_string(),
            grpid: Some(grpid),
            parent_dimensions,
            attributes: Default::default(),
            dimensions: Default::default(),
            groups: Default::default(),
            variables: Default::default(),
        };
        self.groups.insert(name.to_string(), g);
        Ok(self.groups.get_mut(name).unwrap())
    }

    /// Asserts all dimensions exists, and gets a copy of these
    /// (will be moved into a Variable)
    fn find_dimensions(&self, dims: &[&str]) -> error::Result<Vec<Dimension>> {
        let (d, e): (Vec<_>, Vec<_>) = dims
            .iter()
            .map(|name| {
                if let Some(x) = self.dimensions.get(*name) {
                    return Ok(x);
                }
                for pdim in self.parent_dimensions.iter().rev() {
                    if let Some(x) = pdim.get(*name) {
                        return Ok(x);
                    }
                }
                Err(*name)
            })
            .partition(Result::is_ok);

        if !e.is_empty() {
            let mut s = String::new();
            s.push_str("dimension(s)");
            for x in e.into_iter() {
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

    /// Create a Variable into the dataset, with no data written into it
    pub fn add_variable<T>(&mut self, name: &str, dims: &[&str]) -> error::Result<&mut Variable>
    where
        T: Numeric,
    {
        if self.variables.get(name).is_some() {
            return Err(error::Error::AlreadyExists("variable".into()));
        }

        let d = self.find_dimensions(dims)?;
        let var = Variable::new(self.grpid.unwrap_or(self.ncid), name, d, T::NCTYPE)?;

        self.variables.insert(name.into(), var);
        Ok(self.variables.get_mut(name).unwrap())
    }

    /// Adds a variable with a basic type of string
    pub fn add_string_variable(
        &mut self,
        name: &str,
        dims: &[&str],
    ) -> error::Result<&mut Variable> {
        if self.variables.get(name).is_some() {
            return Err(error::Error::AlreadyExists("variable".into()));
        }

        let d = self.find_dimensions(dims)?;
        let var = Variable::new(self.grpid.unwrap_or(self.ncid), name, d, NC_STRING)?;

        self.variables.insert(name.into(), var);
        Ok(self.variables.get_mut(name).unwrap())
    }
}
