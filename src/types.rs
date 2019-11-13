//! User derived types

use netcdf_sys::*;

/// Every netcdf type has a size and an id
pub unsafe trait GeneralType {
    /// typeid for this type
    fn id(&self) -> nc_type;
    /// size in bytes of the type
    fn size(&self) -> usize;
}

/// User defined types
#[derive(Debug, Clone)]
pub enum Type {
    /// A number of bytes
    Opaque(Opaque),
    /// A field with values and corresponding mapping
    /// to names
    Enum(Enum),
    /// A collection of several other types
    Compound(Compound),
}

impl Type {
    /// Name of the type
    pub fn name(&self) -> &str {
        match self {
            Self::Opaque(x) => x.name(),
            Self::Enum(x) => x.name(),
            Self::Compound(x) => x.name(),
        }
    }
    /// size in bytes of the type
    pub fn size(&self) -> usize {
        match self {
            Self::Opaque(x) => x.size(),
            Self::Enum(x) => x.size(),
            Self::Compound(x) => x.size(),
        }
    }
}

#[derive(Debug, Clone)]
/// Bytes with no conversion
pub struct Opaque {
    name: String,
    ncid: nc_type,
    xtype: nc_type,
    size: usize,
}

impl Opaque {
    pub(crate) fn new(name: String, ncid: nc_type, xtype: nc_type, size: usize) -> Self {
        Self {
            name,
            ncid,
            xtype,
            size,
        }
    }

    /// Name of the type
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Size in bytes of type
    pub fn size(&self) -> usize {
        self.size
    }
}

#[derive(Debug, Clone)]
/// Mapping of integer and string values
pub struct Enum {
    name: String,
    ncid: nc_type,
    xtype: nc_type,
    size: usize,
}

impl Enum {
    pub(crate) fn new(name: String, ncid: nc_type, xtype: nc_type, size: usize) -> Self {
        Self {
            name,
            ncid,
            xtype,
            size,
        }
    }

    /// Name of the type
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Size in bytes of type
    pub fn size(&self) -> usize {
        self.size
    }
}

#[derive(Debug, Clone)]
/// A collection of types
pub struct Compound {
    name: String,
    ncid: nc_type,
    xtype: nc_type,
    size: usize,
}

impl Compound {
    pub(crate) fn new(name: String, ncid: nc_type, xtype: nc_type, size: usize) -> Self {
        Self {
            name,
            ncid,
            xtype,
            size,
        }
    }

    /// Name of the type
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Size in bytes of type
    pub fn size(&self) -> usize {
        self.size
    }
}
