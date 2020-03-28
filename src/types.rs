//! Contains functions and enums describing variable types

use super::error;
use crate::with_lock;
use netcdf_sys::*;

/// Basic numeric types
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BasicType {
    /// Signed 1 byte integer
    Byte,
    /// Unsigned 1 byte integer
    Ubyte,
    /// Signed 2 byte integer
    Short,
    /// Unsigned 2 byte integer
    Ushort,
    /// Signed 4 byte integer
    Int,
    /// Unsigned 4 byte integer
    Uint,
    /// Signed 8 byte integer
    Int64,
    /// Unsigned 8 byte integer
    Uint64,
    /// Single precision floating point number
    Float,
    /// Double precision floating point number
    Double,
}

#[allow(missing_docs)]
impl BasicType {
    pub fn is_i8(self) -> bool {
        self == Self::Byte
    }
    pub fn is_u8(self) -> bool {
        self == Self::Ubyte
    }
    pub fn is_i16(self) -> bool {
        self == Self::Short
    }
    pub fn is_u16(self) -> bool {
        self == Self::Ushort
    }
    pub fn is_i32(self) -> bool {
        self == Self::Int
    }
    pub fn is_u32(self) -> bool {
        self == Self::Uint
    }
    pub fn is_i64(self) -> bool {
        self == Self::Int64
    }
    pub fn is_u64(self) -> bool {
        self == Self::Uint64
    }
    pub fn is_f32(self) -> bool {
        self == Self::Float
    }
    pub fn is_f64(self) -> bool {
        self == Self::Double
    }
}

/// Description of the variable
#[derive(Debug, Clone, PartialEq)]
pub enum VariableType {
    /// A basic numeric type
    Basic(BasicType),
    /// A string type
    String,
}

impl VariableType {
    /// Get the basic type, if this type is a simple numeric type
    pub fn as_basic(&self) -> Option<BasicType> {
        match self {
            Self::Basic(x) => Some(*x),
            _ => None,
        }
    }
}

#[allow(missing_docs)]
impl VariableType {
    pub fn is_string(&self) -> bool {
        self == &Self::String
    }
    pub fn is_i8(&self) -> bool {
        self.as_basic().map(|x| x.is_i8()).unwrap_or(false)
    }
    pub fn is_u8(&self) -> bool {
        self.as_basic().map(|x| x.is_u8()).unwrap_or(false)
    }
    pub fn is_i16(&self) -> bool {
        self.as_basic().map(|x| x.is_i16()).unwrap_or(false)
    }
    pub fn is_u16(&self) -> bool {
        self.as_basic().map(|x| x.is_u16()).unwrap_or(false)
    }
    pub fn is_i32(&self) -> bool {
        self.as_basic().map(|x| x.is_i32()).unwrap_or(false)
    }
    pub fn is_u32(&self) -> bool {
        self.as_basic().map(|x| x.is_u32()).unwrap_or(false)
    }
    pub fn is_i64(&self) -> bool {
        self.as_basic().map(|x| x.is_i64()).unwrap_or(false)
    }
    pub fn is_u64(&self) -> bool {
        self.as_basic().map(|x| x.is_u64()).unwrap_or(false)
    }
    pub fn is_f32(&self) -> bool {
        self.as_basic().map(|x| x.is_f32()).unwrap_or(false)
    }
    pub fn is_f64(&self) -> bool {
        self.as_basic().map(|x| x.is_f64()).unwrap_or(false)
    }
}

impl VariableType {
    /// Get the variable type from the id
    pub(crate) fn from_id(_ncid: nc_type, xtype: nc_type) -> error::Result<Self> {
        match xtype {
            NC_BYTE => Ok(Self::Basic(BasicType::Byte)),
            NC_UBYTE => Ok(Self::Basic(BasicType::Ubyte)),
            NC_SHORT => Ok(Self::Basic(BasicType::Short)),
            NC_USHORT => Ok(Self::Basic(BasicType::Ushort)),
            NC_INT => Ok(Self::Basic(BasicType::Int)),
            NC_UINT => Ok(Self::Basic(BasicType::Uint)),
            NC_INT64 => Ok(Self::Basic(BasicType::Int64)),
            NC_UINT64 => Ok(Self::Basic(BasicType::Uint64)),
            NC_FLOAT => Ok(Self::Basic(BasicType::Float)),
            NC_DOUBLE => Ok(Self::Basic(BasicType::Double)),
            NC_STRING => Ok(Self::String),
            _ => Err(format!("{} is still an unknown type", xtype).into()),
        }
    }
}

pub(crate) fn all_at_location(
    ncid: nc_type,
) -> error::Result<impl Iterator<Item = error::Result<VariableType>>> {
    let typeids = {
        let mut ntypeids = 0;
        error::checked(with_lock(|| unsafe {
            nc_inq_typeids(ncid, &mut ntypeids, std::ptr::null_mut())
        }))?;
        let mut typeids = vec![0; ntypeids as _];
        error::checked(with_lock(|| unsafe {
            nc_inq_typeids(ncid, std::ptr::null_mut(), typeids.as_mut_ptr())
        }))?;
        typeids
    };
    Ok(typeids
        .into_iter()
        .map(move |x| VariableType::from_id(ncid, x)))
}
