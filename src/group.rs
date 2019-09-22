use super::attribute::AttrValue;
use super::attribute::Attribute;
use super::dimension::Dimension;
use super::error;
use super::variable::{Numeric, Variable};
use netcdf_sys::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Group {
    pub(crate) name: String,
    pub(crate) ncid: nc_type,
    pub(crate) grpid: Option<nc_type>,
    pub(crate) variables: HashMap<String, Variable>,
    pub(crate) attributes: HashMap<String, Attribute>,
    pub(crate) dimensions: HashMap<String, Dimension>,
    pub(crate) sub_groups: HashMap<String, Group>,
}

impl Group {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn variables(&self) -> &HashMap<String, Variable> {
        &self.variables
    }
    pub fn variable_mut(&mut self, name: &str) -> Option<&mut Variable> {
        self.variables.get_mut(name)
    }
    pub fn attributes(&self) -> &HashMap<String, Attribute> {
        &self.attributes
    }
    pub fn attribute_mut(&mut self, name: &str) -> Option<&mut Attribute> {
        self.attributes.get_mut(name)
    }
    pub fn dimensions(&self) -> &HashMap<String, Dimension> {
        &self.dimensions
    }
    pub fn sub_groups(&self) -> &HashMap<String, Group> {
        &self.sub_groups
    }
    pub fn sub_groups_mut(&mut self, name: &str) -> Option<&mut Group> {
        self.sub_groups.get_mut(name)
    }
}

impl Group {
    pub fn add_attribute<T>(&mut self, name: &str, val: T) -> error::Result<()>
    where
        T: Into<AttrValue>,
    {
        let att = Attribute::put(self.grpid.unwrap_or(self.ncid), NC_GLOBAL, name, val.into())?;
        self.attributes.insert(name.to_string().clone(), att);
        Ok(())
    }

    pub fn add_dimension(&mut self, name: &str, len: usize) -> error::Result<&mut Dimension> {
        if self.dimensions.contains_key(name) {
            return Err(format!("Dimension {} already exists", name).into());
        }

        self.dimensions.insert(
            name.into(),
            Dimension::new(self.grpid.unwrap_or(self.ncid), name, len)?,
        );

        Ok(self.dimensions.get_mut(name).unwrap())
    }

    /// Create a Variable into the dataset, without writting any data into it.
    pub fn add_variable<T>(&mut self, name: &str, dims: &[&str]) -> error::Result<&mut Variable>
    where
        T: Numeric,
    {
        if self.variables.get(name).is_some() {
            return Err(format!("variable {} already exists", name).into());
        }

        // Assert all dimensions exists, and get &[&Dimension]
        let (d, e): (Vec<_>, Vec<_>) = dims
            .iter()
            .map(|x| self.dimensions.get(*x).ok_or(*x))
            .partition(Result::is_ok);

        if !e.is_empty() {
            return Err(format!(
                "Dimensions not found: {:?}",
                e.into_iter().map(Result::unwrap_err).collect::<Vec<_>>()
            )
            .into());
        }

        let d = d.into_iter().map(Result::unwrap).collect::<Vec<_>>();
        let var = Variable::new(self.grpid.unwrap_or(self.ncid), name, &d, T::NCTYPE)?;

        self.variables.insert(name.into(), var);

        Ok(self.variables.get_mut(name).unwrap())
    }
}
