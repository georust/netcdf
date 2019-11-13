//! User derived types

use netcdf_sys::*;

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
    /// a variable length array
    VariableArray(VariableArray),
    /// A simple type
    Simple(Simple),
    /// A string type
    String,
}

impl Type {
    /// Name of the type
    pub fn name(&self) -> &str {
        match self {
            Self::Opaque(x) => x.name(),
            Self::Enum(x) => x.name(),
            Self::Compound(x) => x.name(),
            Self::VariableArray(x) => x.name(),
            Self::Simple(x) => x.name(),
            Self::String => "string",
        }
    }
    /// size in bytes of the type
    pub fn size(&self) -> Option<usize> {
        match self {
            Self::Opaque(x) => Some(x.size()),
            Self::Enum(x) => Some(x.size()),
            Self::Compound(x) => Some(x.size()),
            Self::VariableArray(_) | Self::String => None,
            Self::Simple(x) => Some(x.size()),
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

#[derive(Debug, Clone)]
/// Variable length array
pub struct VariableArray {
    pub(crate) ncid: nc_type,
    pub(crate) varid: nc_type,
    pub(crate) name: String,
}

impl VariableArray {
    /// Name of this array
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
#[allow(missing_docs)]
/// A basic netcdf type
pub enum Simple {
    Schar,
    Uchar,
    Short,
    Ushort,
    Int,
    Uint,
    Longlong,
    Ulonglong,
    Float,
    Double,
}

impl Simple {
    fn name(&self) -> &str {
        match self {
            Self::Schar => "i8",
            Self::Uchar => "u8",
            Self::Short => "i16",
            Self::Ushort => "u16",
            Self::Int => "i32",
            Self::Uint => "u32",
            Self::Longlong => "i64",
            Self::Ulonglong => "u64",
            Self::Float => "f32",
            Self::Double => "f64",
        }
    }
    fn size(&self) -> usize {
        use std::mem::size_of;
        match self {
            Self::Schar | Self::Uchar => size_of::<u8>(),
            Self::Short | Self::Ushort => size_of::<u16>(),
            Self::Int | Self::Uint | Self::Float => size_of::<u32>(),
            Self::Longlong | Self::Ulonglong | Self::Double => size_of::<u64>(),
        }
    }
}
