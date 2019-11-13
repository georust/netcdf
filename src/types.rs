//! User derived types

use super::error;
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

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Opaque(x), Self::Opaque(y)) => {
                let mut equal = 0;
                let e;
                unsafe {
                    e = error::checked(nc_inq_type_equal(
                        x.ncid, x.xtype, y.ncid, y.xtype, &mut equal,
                    ));
                }
                !(e.is_err() || equal == 0)
            }
            (Self::Enum(x), Self::Enum(y)) => {
                let mut equal = 0;
                let e;
                unsafe {
                    e = error::checked(nc_inq_type_equal(
                        x.ncid, x.xtype, y.ncid, y.xtype, &mut equal,
                    ));
                }
                !(e.is_err() || equal == 0)
            }
            (Self::Compound(x), Self::Compound(y)) => {
                let mut equal = 0;
                let e;
                unsafe {
                    e = error::checked(nc_inq_type_equal(
                        x.ncid, x.xtype, y.ncid, y.xtype, &mut equal,
                    ));
                }
                !(e.is_err() || equal == 0)
            }
            (Self::VariableArray(x), Self::VariableArray(y)) => {
                let mut equal = 0;
                let e;
                unsafe {
                    e = error::checked(nc_inq_type_equal(
                        x.ncid, x.varid, y.ncid, y.varid, &mut equal,
                    ));
                }
                !(e.is_err() || equal == 0)
            }
            (Self::Simple(x), Self::Simple(y)) => nc_type::from(x) == nc_type::from(y),
            (Self::String, Self::String) => true,
            _ => false,
        }
    }
}

impl Eq for Type {}

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
    pub(crate) fn id(&self) -> nc_type {
        match self {
            Self::Opaque(o) => o.xtype,
            Self::Enum(e) => e.xtype,
            Self::Compound(c) => c.xtype,
            Self::VariableArray(v) => v.varid,
            Self::String => NC_STRING,
            Self::Simple(x) => nc_type::from(x),
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

impl std::convert::TryFrom<nc_type> for Simple {
    type Error = ();

    fn try_from(value: nc_type) -> Result<Self, Self::Error> {
        match value {
            NC_BYTE => Ok(Self::Schar),
            NC_UBYTE => Ok(Self::Uchar),
            NC_SHORT => Ok(Self::Short),
            NC_USHORT => Ok(Self::Ushort),
            NC_INT => Ok(Self::Int),
            NC_UINT => Ok(Self::Uint),
            NC_INT64 => Ok(Self::Longlong),
            NC_UINT64 => Ok(Self::Ulonglong),
            NC_FLOAT => Ok(Self::Float),
            NC_DOUBLE => Ok(Self::Double),
            _ => Err(()),
        }
    }
}
impl std::convert::From<&Simple> for nc_type {
    fn from(value: &Simple) -> Self {
        match value {
            Simple::Schar => NC_BYTE,
            Simple::Uchar => NC_UBYTE,
            Simple::Short => NC_SHORT,
            Simple::Ushort => NC_USHORT,
            Simple::Int => NC_INT,
            Simple::Uint => NC_UINT,
            Simple::Longlong => NC_INT64,
            Simple::Ulonglong => NC_UINT64,
            Simple::Float => NC_FLOAT,
            Simple::Double => NC_DOUBLE,
        }
    }
}
