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

#[derive(Clone, Debug)]
/// A set of bytes which with unspecified endianess
pub struct OpaqueType {
    ncid: nc_type,
    id: nc_type,
}

impl OpaqueType {
    /// Get the name of this opaque type
    pub fn name(&self) -> String {
        let mut name = [0_u8; NC_MAX_NAME as usize + 1];
        error::checked(super::with_lock(|| unsafe {
            nc_inq_opaque(
                self.ncid,
                self.id,
                name.as_mut_ptr() as *mut _,
                std::ptr::null_mut(),
            )
        }))
        .unwrap();

        let pos = name.iter().position(|&x| x == 0).unwrap_or(name.len());
        String::from_utf8(name[..pos].to_vec()).unwrap()
    }
    /// Number of bytes this type occupies
    pub fn size(&self) -> usize {
        let mut numbytes = 0;
        error::checked(super::with_lock(|| unsafe {
            nc_inq_opaque(self.ncid, self.id, std::ptr::null_mut(), &mut numbytes)
        }))
        .unwrap();
        numbytes
    }
    pub(crate) fn add(location: nc_type, name: &str, size: usize) -> error::Result<Self> {
        let cname = super::utils::short_name_to_bytes(name)?;
        let mut id = 0;
        error::checked(super::with_lock(|| unsafe {
            nc_def_opaque(location, size, cname.as_ptr() as *const _, &mut id)
        }))?;

        Ok(Self { ncid: location, id })
    }
}

/// Type of variable length
#[derive(Debug, Clone)]
pub struct VlenType {
    ncid: nc_type,
    id: nc_type,
}

impl VlenType {
    /// Name of the type
    pub fn name(&self) -> String {
        let mut name = [0_u8; NC_MAX_NAME as usize + 1];
        error::checked(super::with_lock(|| unsafe {
            nc_inq_vlen(
                self.ncid,
                self.id,
                name.as_mut_ptr() as *mut _,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        }))
        .unwrap();

        let pos = name.iter().position(|&x| x == 0).unwrap_or(name.len());
        String::from_utf8(name[..pos].to_vec()).unwrap()
    }

    pub(crate) fn add<T>(location: nc_type, name: &str) -> error::Result<Self>
    where
        T: super::Numeric,
    {
        let cname = super::utils::short_name_to_bytes(name)?;
        let mut id = 0;
        error::checked(super::with_lock(|| unsafe {
            nc_def_vlen(location, cname.as_ptr() as *const _, T::NCTYPE, &mut id)
        }))?;

        Ok(Self { ncid: location, id })
    }

    /// Internal type
    pub fn typ(&self) -> BasicType {
        let mut bastyp = 0;
        error::checked(super::with_lock(|| unsafe {
            nc_inq_vlen(
                self.ncid,
                self.id,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut bastyp,
            )
        }))
        .unwrap();

        match bastyp {
            NC_BYTE => BasicType::Byte,
            NC_UBYTE => BasicType::Ubyte,
            NC_SHORT => BasicType::Short,
            NC_USHORT => BasicType::Ushort,
            NC_INT => BasicType::Int,
            NC_UINT => BasicType::Uint,
            NC_INT64 => BasicType::Int64,
            NC_UINT64 => BasicType::Uint64,
            NC_FLOAT => BasicType::Float,
            NC_DOUBLE => BasicType::Double,
            _ => panic!("Did not expect typeid {} in this context", bastyp),
        }
    }
}

#[derive(Debug, Clone)]
/// Multiple string values stored as integer type
pub struct EnumType {
    ncid: nc_type,
    id: nc_type,
}

impl EnumType {
    pub(crate) fn add<T: super::Numeric>(
        ncid: nc_type,
        name: &str,
        mappings: &[(&str, T)],
    ) -> error::Result<EnumType> {
        let cname = super::utils::short_name_to_bytes(name)?;
        let mut id = 0;
        error::checked(super::with_lock(|| unsafe {
            nc_def_enum(ncid, T::NCTYPE, cname.as_ptr() as *const _, &mut id)
        }))?;

        for (name, val) in mappings {
            let cname = super::utils::short_name_to_bytes(name)?;
            error::checked(super::with_lock(|| unsafe {
                nc_insert_enum(
                    ncid,
                    id,
                    cname.as_ptr() as *const _,
                    val as *const T as *const _,
                )
            }))?;
        }

        Ok(Self { ncid, id })
    }

    /// Get the base type of the enum
    pub fn typ(&self) -> BasicType {
        let mut typ = 0;
        error::checked(super::with_lock(|| unsafe {
            nc_inq_enum(
                self.ncid,
                self.id,
                std::ptr::null_mut(),
                &mut typ,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        }))
        .unwrap();
        match typ {
            NC_BYTE => BasicType::Byte,
            NC_UBYTE => BasicType::Ubyte,
            NC_SHORT => BasicType::Short,
            NC_USHORT => BasicType::Ushort,
            NC_INT => BasicType::Int,
            NC_UINT => BasicType::Uint,
            NC_INT64 => BasicType::Int64,
            NC_UINT64 => BasicType::Uint64,
            NC_FLOAT => BasicType::Float,
            NC_DOUBLE => BasicType::Double,
            _ => panic!("Did not expect typeid {} in this context", typ),
        }
    }

    /// Get a single member from an index
    ///
    /// # Safety
    /// Does not check type of enum
    unsafe fn member_at<T: super::Numeric>(&self, idx: usize) -> error::Result<(String, T)> {
        let mut name = [0_u8; NC_MAX_NAME as usize + 1];
        let mut t = std::mem::MaybeUninit::<T>::uninit();
        super::with_lock(|| {
            nc_inq_enum_member(
                self.ncid,
                self.id,
                idx as _,
                name.as_mut_ptr() as *mut _,
                t.as_mut_ptr() as *mut _,
            )
        });

        let pos = name.iter().position(|&x| x == 0).unwrap_or(name.len());
        let name = String::from_utf8(name[..pos].to_vec()).unwrap();
        Ok((name, t.assume_init()))
    }

    /// Get all members of the enum
    pub fn members<'f, T: super::Numeric>(
        &'f self,
    ) -> error::Result<impl Iterator<Item = (String, T)> + 'f> {
        let mut typ = 0;
        let mut nummembers = 0;
        error::checked(super::with_lock(|| unsafe {
            nc_inq_enum(
                self.ncid,
                self.id,
                std::ptr::null_mut(),
                &mut typ,
                std::ptr::null_mut(),
                &mut nummembers,
            )
        }))
        .unwrap();
        if typ != T::NCTYPE {
            return Err(error::Error::TypeMismatch);
        }

        Ok((0..nummembers)
            .into_iter()
            .map(move |idx| unsafe { self.member_at::<T>(idx) }.unwrap()))
    }

    /// Name of the type
    pub fn name(&self) -> String {
        let mut name = [0_u8; NC_MAX_NAME as usize + 1];
        error::checked(super::with_lock(|| unsafe {
            nc_inq_enum(
                self.ncid,
                self.id,
                name.as_mut_ptr() as *mut _,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        }))
        .unwrap();

        let pos = name.iter().position(|&x| x == 0).unwrap_or(name.len());
        String::from_utf8(name[..pos].to_vec()).unwrap()
    }

    /// Get the name from the enum value
    pub fn name_from_value(&self, value: i64) -> Option<String> {
        let mut cname = [0_u8; NC_MAX_NAME as usize + 1];
        let e = super::with_lock(|| unsafe {
            nc_inq_enum_ident(self.ncid, self.id, value, cname.as_mut_ptr() as *mut _)
        });
        if e == NC_EINVAL {
            return None;
        }

        error::checked(e).unwrap();

        let pos = cname.iter().position(|&x| x == 0).unwrap_or(cname.len());
        Some(String::from_utf8(cname[..pos].to_vec()).unwrap())
    }
}

/// Description of the variable
#[derive(Debug, Clone)]
pub enum VariableType {
    /// A basic numeric type
    Basic(BasicType),
    /// A string type
    String,
    /// Some bytes
    Opaque(OpaqueType),
    /// Variable length array
    Vlen(VlenType),
    /// Enum type
    Enum(EnumType),
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
        match self {
            Self::String => true,
            _ => false,
        }
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
