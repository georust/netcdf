//! Contains functions and enums describing variable types

use super::error;
use crate::with_lock;
use netcdf_sys::*;
use std::convert::TryInto;

/// Basic numeric types
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BasicType {
    /// Signed 1 byte integer
    Byte,
    /// ISO/ASCII character
    Char,
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

impl BasicType {
    /// Size of the type in bytes
    fn size(self) -> usize {
        match self {
            Self::Byte | Self::Ubyte | Self::Char => 1,
            Self::Short | Self::Ushort => 2,
            Self::Int | Self::Uint | Self::Float => 4,
            Self::Int64 | Self::Uint64 | Self::Double => 8,
        }
    }
    /// `nc_type` of the type
    pub(crate) fn id(self) -> nc_type {
        use super::NcPutGet;
        match self {
            Self::Byte => i8::NCTYPE,
            Self::Char => NC_CHAR,
            Self::Ubyte => u8::NCTYPE,
            Self::Short => i16::NCTYPE,
            Self::Ushort => u16::NCTYPE,
            Self::Int => i32::NCTYPE,
            Self::Uint => u32::NCTYPE,
            Self::Int64 => i64::NCTYPE,
            Self::Uint64 => u64::NCTYPE,
            Self::Float => f32::NCTYPE,
            Self::Double => f64::NCTYPE,
        }
    }

    /// `rusty` name of the type
    pub fn name(self) -> &'static str {
        match self {
            Self::Byte => "i8",
            Self::Char => "char",
            Self::Ubyte => "u8",
            Self::Short => "i16",
            Self::Ushort => "u16",
            Self::Int => "i32",
            Self::Uint => "u32",
            Self::Int64 => "i64",
            Self::Uint64 => "u64",
            Self::Float => "f32",
            Self::Double => "f64",
        }
    }
}

#[allow(missing_docs)]
impl BasicType {
    pub fn is_i8(self) -> bool {
        self == Self::Byte
    }
    pub fn is_char(self) -> bool {
        self == Self::Char
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
                name.as_mut_ptr().cast(),
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
        let name = super::utils::short_name_to_bytes(name)?;
        let mut id = 0;
        error::checked(super::with_lock(|| unsafe {
            nc_def_opaque(location, size, name.as_ptr().cast(), &mut id)
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
                name.as_mut_ptr().cast(),
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
        T: super::NcPutGet,
    {
        let name = super::utils::short_name_to_bytes(name)?;
        let mut id = 0;
        error::checked(super::with_lock(|| unsafe {
            nc_def_vlen(location, name.as_ptr().cast(), T::NCTYPE, &mut id)
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
            _ => panic!("Did not expect typeid {bastyp} in this context"),
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
    pub(crate) fn add<T: super::NcPutGet>(
        ncid: nc_type,
        name: &str,
        mappings: &[(&str, T)],
    ) -> error::Result<Self> {
        let name = super::utils::short_name_to_bytes(name)?;
        let mut id = 0;
        error::checked(super::with_lock(|| unsafe {
            nc_def_enum(ncid, T::NCTYPE, name.as_ptr().cast(), &mut id)
        }))?;

        for (name, val) in mappings {
            let name = super::utils::short_name_to_bytes(name)?;
            error::checked(super::with_lock(|| unsafe {
                nc_insert_enum(ncid, id, name.as_ptr().cast(), (val as *const T).cast())
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
            _ => panic!("Did not expect typeid {typ} in this context"),
        }
    }

    /// Get a single member from an index
    ///
    /// # Safety
    /// Does not check type of enum
    unsafe fn member_at<T: super::NcPutGet>(&self, idx: usize) -> error::Result<(String, T)> {
        let mut name = [0_u8; NC_MAX_NAME as usize + 1];
        let mut t = std::mem::MaybeUninit::<T>::uninit();
        let idx = idx.try_into()?;
        super::with_lock(|| {
            nc_inq_enum_member(
                self.ncid,
                self.id,
                idx,
                name.as_mut_ptr().cast(),
                t.as_mut_ptr().cast(),
            )
        });

        let pos = name.iter().position(|&x| x == 0).unwrap_or(name.len());
        let name = String::from_utf8(name[..pos].to_vec()).unwrap();
        Ok((name, t.assume_init()))
    }

    /// Get all members of the enum
    pub fn members<T: super::NcPutGet>(
        &self,
    ) -> error::Result<impl Iterator<Item = (String, T)> + '_> {
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

        Ok((0..nummembers).map(move |idx| unsafe { self.member_at::<T>(idx) }.unwrap()))
    }

    /// Name of the type
    pub fn name(&self) -> String {
        let mut name = [0_u8; NC_MAX_NAME as usize + 1];
        error::checked(super::with_lock(|| unsafe {
            nc_inq_enum(
                self.ncid,
                self.id,
                name.as_mut_ptr().cast(),
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
        let mut name = [0_u8; NC_MAX_NAME as usize + 1];
        let e = super::with_lock(|| unsafe {
            nc_inq_enum_ident(self.ncid, self.id, value, name.as_mut_ptr().cast())
        });
        if e == NC_EINVAL {
            return None;
        }

        error::checked(e).unwrap();

        let pos = name.iter().position(|&x| x == 0).unwrap_or(name.len());
        Some(String::from_utf8(name[..pos].to_vec()).unwrap())
    }

    /// Size in bytes of this type
    fn size(&self) -> usize {
        self.typ().size()
    }
}

/// A type consisting of other types
#[derive(Debug, Clone)]
pub struct CompoundType {
    ncid: nc_type,
    id: nc_type,
}

impl CompoundType {
    pub(crate) fn add(ncid: nc_type, name: &str) -> error::Result<CompoundBuilder> {
        let name = super::utils::short_name_to_bytes(name)?;

        Ok(CompoundBuilder {
            ncid,
            name,
            size: 0,
            comp: Vec::new(),
        })
    }

    /// Size in bytes of this type
    fn size(&self) -> usize {
        let mut size = 0;
        error::checked(super::with_lock(|| unsafe {
            nc_inq_compound(
                self.ncid,
                self.id,
                std::ptr::null_mut(),
                &mut size,
                std::ptr::null_mut(),
            )
        }))
        .unwrap();
        size
    }

    /// Get the name of this type
    pub fn name(&self) -> String {
        let mut name = [0_u8; NC_MAX_NAME as usize + 1];
        error::checked(super::with_lock(|| unsafe {
            nc_inq_compound(
                self.ncid,
                self.id,
                name.as_mut_ptr().cast(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        }))
        .unwrap();

        let pos = name.iter().position(|&x| x == 0).unwrap_or(name.len());
        String::from_utf8(name[..pos].to_vec()).unwrap()
    }

    /// Get the fields of the compound
    pub fn fields(&self) -> impl Iterator<Item = CompoundField> {
        let ncid = self.ncid;
        let parent_id = self.id;

        let mut nfields = 0;
        error::checked(super::with_lock(|| unsafe {
            nc_inq_compound_nfields(ncid, parent_id, &mut nfields)
        }))
        .unwrap();

        (0..nfields).map(move |x| CompoundField {
            ncid,
            parent: parent_id,
            id: x,
        })
    }
}

/// Subfield of a compound
pub struct CompoundField {
    ncid: nc_type,
    parent: nc_type,
    id: usize,
}

impl CompoundField {
    /// Name of the compound field
    pub fn name(&self) -> String {
        let mut name = [0_u8; NC_MAX_NAME as usize + 1];
        let idx = self.id.try_into().unwrap();
        error::checked(super::with_lock(|| unsafe {
            nc_inq_compound_fieldname(self.ncid, self.parent, idx, name.as_mut_ptr().cast())
        }))
        .unwrap();

        let pos = name.iter().position(|&x| x == 0).unwrap_or(name.len());
        String::from_utf8(name[..pos].to_vec()).unwrap()
    }

    /// type of the field
    pub fn typ(&self) -> VariableType {
        let mut typ = 0;
        let id = self.id.try_into().unwrap();
        error::checked(super::with_lock(|| unsafe {
            nc_inq_compound_fieldtype(self.ncid, self.parent, id, &mut typ)
        }))
        .unwrap();

        VariableType::from_id(self.ncid, typ).unwrap()
    }

    /// Offset in bytes of this field in the compound type
    pub fn offset(&self) -> usize {
        let mut offset = 0;
        let id = self.id.try_into().unwrap();
        error::checked(super::with_lock(|| unsafe {
            nc_inq_compound_field(
                self.ncid,
                self.parent,
                id,
                std::ptr::null_mut(),
                &mut offset,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        }))
        .unwrap();

        offset
    }

    /// Get dimensionality of this compound field
    pub fn dimensions(&self) -> Option<Vec<usize>> {
        let mut num_dims = 0;
        let id = self.id.try_into().unwrap();
        error::checked(super::with_lock(|| unsafe {
            nc_inq_compound_fieldndims(self.ncid, self.parent, id, &mut num_dims)
        }))
        .unwrap();

        if num_dims == 0 {
            return None;
        }

        let mut dims = vec![0; num_dims.try_into().unwrap()];
        error::checked(super::with_lock(|| unsafe {
            nc_inq_compound_fielddim_sizes(self.ncid, self.parent, id, dims.as_mut_ptr())
        }))
        .unwrap();

        Some(dims.iter().map(|&x| x.try_into().unwrap()).collect())
    }
}

/// A builder for a compound type
#[must_use]
pub struct CompoundBuilder {
    ncid: nc_type,
    name: [u8; NC_MAX_NAME as usize + 1],
    size: usize,
    comp: Vec<(
        VariableType,
        [u8; NC_MAX_NAME as usize + 1],
        Option<Vec<i32>>,
    )>,
}

impl CompoundBuilder {
    /// Add a type to the compound
    pub fn add_type(&mut self, name: &str, var: &VariableType) -> error::Result<&mut Self> {
        self.comp
            .push((var.clone(), super::utils::short_name_to_bytes(name)?, None));

        self.size += var.size();
        Ok(self)
    }

    /// Add a basic numeric type
    pub fn add<T: super::NcPutGet>(&mut self, name: &str) -> error::Result<&mut Self> {
        let var = VariableType::from_id(self.ncid, T::NCTYPE)?;
        self.add_type(name, &var)
    }

    /// Add an array of a basic type
    pub fn add_array<T: super::NcPutGet>(
        &mut self,
        name: &str,
        dims: &[usize],
    ) -> error::Result<&mut Self> {
        let var = VariableType::from_id(self.ncid, T::NCTYPE)?;
        self.add_array_type(name, &var, dims)
    }

    /// Add a type as an array
    pub fn add_array_type(
        &mut self,
        name: &str,
        var: &VariableType,
        dims: &[usize],
    ) -> error::Result<&mut Self> {
        self.comp.push((
            var.clone(),
            super::utils::short_name_to_bytes(name)?,
            Some(dims.iter().map(|&x| x.try_into().unwrap()).collect()),
        ));

        self.size += var.size() * dims.iter().product::<usize>();
        Ok(self)
    }

    /// Finalize the compound type
    pub fn build(self) -> error::Result<CompoundType> {
        let mut id = 0;
        error::checked(super::with_lock(|| unsafe {
            nc_def_compound(self.ncid, self.size, self.name.as_ptr().cast(), &mut id)
        }))?;

        let mut offset = 0;
        for (typ, name, dims) in &self.comp {
            match dims {
                None => {
                    error::checked(super::with_lock(|| unsafe {
                        nc_insert_compound(self.ncid, id, name.as_ptr().cast(), offset, typ.id())
                    }))?;
                    offset += typ.size();
                }
                Some(dims) => {
                    let dimlen = dims.len().try_into().unwrap();
                    error::checked(super::with_lock(|| unsafe {
                        nc_insert_array_compound(
                            self.ncid,
                            id,
                            name.as_ptr().cast(),
                            offset,
                            typ.id(),
                            dimlen,
                            dims.as_ptr(),
                        )
                    }))?;
                    offset += typ.size()
                        * dims
                            .iter()
                            .map(|x: &i32| -> usize { (*x).try_into().unwrap() })
                            .product::<usize>();
                }
            }
        }

        Ok(CompoundType {
            ncid: self.ncid,
            id,
        })
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
    /// Compound type
    Compound(CompoundType),
}

impl VariableType {
    /// Get the basic type, if this type is a simple numeric type
    pub fn as_basic(&self) -> Option<BasicType> {
        match self {
            Self::Basic(x) => Some(*x),
            _ => None,
        }
    }

    /// Size in bytes of the type
    pub(crate) fn size(&self) -> usize {
        match self {
            Self::Basic(b) => b.size(),
            Self::String => panic!("A string does not have a defined size"),
            Self::Enum(e) => e.size(),
            Self::Opaque(o) => o.size(),
            Self::Vlen(_) => panic!("A variable length array does not have a defined size"),
            Self::Compound(c) => c.size(),
        }
    }

    /// Id of this type
    pub(crate) fn id(&self) -> nc_type {
        match self {
            Self::Basic(b) => b.id(),
            Self::String => NC_STRING,
            Self::Enum(e) => e.id,
            Self::Opaque(o) => o.id,
            Self::Vlen(v) => v.id,
            Self::Compound(c) => c.id,
        }
    }

    /// Get the name of the type. The basic numeric types will
    /// have `rusty` names (u8/i32/f64/string)
    pub fn name(&self) -> String {
        match self {
            Self::Basic(b) => b.name().into(),
            Self::String => "string".into(),
            Self::Enum(e) => e.name(),
            Self::Opaque(o) => o.name(),
            Self::Vlen(v) => v.name(),
            Self::Compound(c) => c.name(),
        }
    }
}

#[allow(missing_docs)]
impl VariableType {
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String)
    }
    pub fn is_i8(&self) -> bool {
        self.as_basic().map_or(false, BasicType::is_i8)
    }
    pub fn is_u8(&self) -> bool {
        self.as_basic().map_or(false, BasicType::is_u8)
    }
    pub fn is_i16(&self) -> bool {
        self.as_basic().map_or(false, BasicType::is_i16)
    }
    pub fn is_u16(&self) -> bool {
        self.as_basic().map_or(false, BasicType::is_u16)
    }
    pub fn is_i32(&self) -> bool {
        self.as_basic().map_or(false, BasicType::is_i32)
    }
    pub fn is_u32(&self) -> bool {
        self.as_basic().map_or(false, BasicType::is_u32)
    }
    pub fn is_i64(&self) -> bool {
        self.as_basic().map_or(false, BasicType::is_i64)
    }
    pub fn is_u64(&self) -> bool {
        self.as_basic().map_or(false, BasicType::is_u64)
    }
    pub fn is_f32(&self) -> bool {
        self.as_basic().map_or(false, BasicType::is_f32)
    }
    pub fn is_f64(&self) -> bool {
        self.as_basic().map_or(false, BasicType::is_f64)
    }
}

impl VariableType {
    /// Get the variable type from the id
    pub(crate) fn from_id(ncid: nc_type, xtype: nc_type) -> error::Result<Self> {
        match xtype {
            NC_CHAR => Ok(Self::Basic(BasicType::Char)),
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
            xtype => {
                let mut base_xtype = 0;
                error::checked(super::with_lock(|| unsafe {
                    nc_inq_user_type(
                        ncid,
                        xtype,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        &mut base_xtype,
                    )
                }))?;
                match base_xtype {
                    NC_VLEN => Ok(VlenType { ncid, id: xtype }.into()),
                    NC_OPAQUE => Ok(OpaqueType { ncid, id: xtype }.into()),
                    NC_ENUM => Ok(EnumType { ncid, id: xtype }.into()),
                    NC_COMPOUND => Ok(CompoundType { ncid, id: xtype }.into()),
                    _ => panic!("Unexpected base type: {base_xtype}"),
                }
            }
        }
    }
}

pub(crate) fn all_at_location(
    ncid: nc_type,
) -> error::Result<impl Iterator<Item = error::Result<VariableType>>> {
    let typeids = {
        let mut num_typeids = 0;
        error::checked(with_lock(|| unsafe {
            nc_inq_typeids(ncid, &mut num_typeids, std::ptr::null_mut())
        }))?;
        let mut typeids = vec![0; num_typeids.try_into()?];
        error::checked(with_lock(|| unsafe {
            nc_inq_typeids(ncid, std::ptr::null_mut(), typeids.as_mut_ptr())
        }))?;
        typeids
    };
    Ok(typeids
        .into_iter()
        .map(move |x| VariableType::from_id(ncid, x)))
}

impl From<CompoundType> for VariableType {
    fn from(v: CompoundType) -> Self {
        Self::Compound(v)
    }
}

impl From<BasicType> for VariableType {
    fn from(v: BasicType) -> Self {
        Self::Basic(v)
    }
}

impl From<EnumType> for VariableType {
    fn from(v: EnumType) -> Self {
        Self::Enum(v)
    }
}

impl From<VlenType> for VariableType {
    fn from(v: VlenType) -> Self {
        Self::Vlen(v)
    }
}

impl From<OpaqueType> for VariableType {
    fn from(v: OpaqueType) -> Self {
        Self::Opaque(v)
    }
}
