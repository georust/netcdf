//! Types found in `netCDF` files
use netcdf_sys::{
    nc_type, NC_BYTE, NC_CHAR, NC_COMPOUND, NC_DOUBLE, NC_ENUM, NC_FLOAT, NC_INT, NC_INT64,
    NC_MAX_NAME, NC_OPAQUE, NC_SHORT, NC_STRING, NC_UBYTE, NC_UINT, NC_UINT64, NC_USHORT, NC_VLEN,
};

use crate::{error::Result, utils::checked_with_lock};

/// This trait allows reading and writing basic and user defined types.
///
/// Supports basic types ([`i8`], [`u8`], [`i16`], ..., [`f32`], [`f64`]) and user-defined types.
///
/// Prefer deriving using [`NcType`][crate::NcType] when working with
/// user defined types. With the `derive` feature enabled for this crate one can
/// easily define types for reading and writing to and from `netCDF` files.
/// # Example (derive macro)
/// ```rust
/// # #[cfg(feature = "derive")]
/// #[repr(C)]
/// #[derive(netcdf::NcType, Debug, Copy, Clone)]
/// struct Foo {
///     a: i32,
///     b: u32,
/// }
/// # #[cfg(feature = "derive")]
/// #[repr(u32)]
/// #[derive(netcdf::NcType, Debug, Copy, Clone)]
/// enum Bar {
///     Egg = 3,
///     Milk,
/// }
/// # #[cfg(feature = "derive")]
/// #[repr(C)]
/// #[derive(netcdf::NcType, Debug, Copy, Clone)]
/// struct FooBar {
///     foo: Foo,
///     bar: Bar,
/// }
/// # #[cfg(feature = "derive")]
/// #[repr(C)]
/// #[derive(netcdf::NcType, Debug, Copy, Clone)]
/// struct Arrayed {
///     a: [[u8; 3]; 5],
///     b: i8,
/// }
/// # #[cfg(feature = "derive")]
/// #[repr(C)]
/// #[derive(netcdf::NcType, Debug, Copy, Clone)]
/// #[netcdf(rename = "myname")]
/// struct Renamed {
///     #[netcdf(rename = "orange")]
///     a: u64,
///     #[netcdf(rename = "apple")]
///     b: i64,
/// }
/// ```
/// # Examples (advanced)
/// The following examples illustrates how to implement more advanced types.
/// They are not included in this crate since they either have difficulties
/// interacting with `Drop` (vlen, string) or they include design choices
/// such as naming (char) or type name (opaque, enum).
///
/// ## Char type
/// Reading of an `netcdf_sys::NC_CHAR` can not be done by using `i8` or `u8` as
/// such types are not considered text. The below snippet can be used to define
/// a type which will read this type.
/// ```rust
/// # use netcdf::types::*;
/// #[repr(transparent)]
/// #[derive(Copy, Clone)]
/// struct NcChar(i8);
/// unsafe impl NcTypeDescriptor for NcChar {
///     fn type_descriptor() -> NcVariableType {
///         NcVariableType::Char
///     }
/// }
/// ```
/// ## Opaque type
/// ```rust
/// # use netcdf::types::*;
/// #[repr(transparent)]
/// #[derive(Copy, Clone)]
/// struct Opaque([u8; 16]);
/// unsafe impl NcTypeDescriptor for Opaque {
///     fn type_descriptor() -> NcVariableType {
///         NcVariableType::Opaque(OpaqueType {
///             name: "Opaque".to_owned(),
///             size: std::mem::size_of::<Opaque>()
///         })
///     }
/// }
/// ```
/// ## Vlen type
/// This type *must* match [`netcdf_sys::nc_vlen_t`]. Be aware that reading using this
/// type means the memory is backed by `netCDF` and should be
/// freed using [`netcdf_sys::nc_free_vlen`] or [`netcdf_sys::nc_free_vlens`]
/// to avoid memory leaks.
/// ```rust
/// # use netcdf::types::*;
/// #[repr(C)]
/// struct Vlen {
///     len: usize,
///     p: *const u8,
/// }
/// unsafe impl NcTypeDescriptor for Vlen {
///     fn type_descriptor() -> NcVariableType {
///         NcVariableType::Vlen(VlenType {
///             name: "Vlen".to_owned(),
///             basetype: Box::new(NcVariableType::Int(IntType::U8)),
///         })
///     }
/// }
/// ```
/// ## String type
/// String types must be freed using [`netcdf_sys::nc_free_string`].
/// ```rust
/// # use netcdf::types::*;
/// #[repr(transparent)]
/// struct NcString(*mut std::ffi::c_char);
/// unsafe impl NcTypeDescriptor for NcString {
///     fn type_descriptor() -> NcVariableType {
///         NcVariableType::String
///     }
/// }
/// ```
///
/// # Safety
/// Below is a list of things to keep in mind when implementing:
/// * `Drop` interaction when types are instantiated (reading)
/// * Padding bytes in the struct
/// * Alignment of members in a struct
/// * Endianness (for opaque structs)
/// * Overlapping compound members
/// * Duplicate enum/compound names
/// * Duplicate enum values
pub unsafe trait NcTypeDescriptor {
    /// Description of the type
    fn type_descriptor() -> NcVariableType;
    #[doc(hidden)]
    /// This is here to allow e.g. [u8; 4] in compounds and should
    /// be considered a hack.
    /// This item is ignored in non-compounds and will lead to confusing
    /// error messages if used in non-compound types.
    const ARRAY_ELEMENTS: ArrayElements = ArrayElements::None;
}

#[derive(Clone, PartialEq, Eq, Debug)]
/// A `netCDF` type
///
/// This enum contains all variants of types allowed by `netCDF`.
pub enum NcVariableType {
    /// A compound struct
    Compound(CompoundType),
    /// A bag of bytes
    Opaque(OpaqueType),
    /// An enumeration of names/values
    Enum(EnumType),
    /// Ragged array
    Vlen(VlenType),
    /// String type
    String,
    /// Integer type
    Int(IntType),
    /// Floating type
    Float(FloatType),
    /// Char type
    Char,
}

impl NcVariableType {
    /// Size (in bytes) of the type in memory
    pub fn size(&self) -> usize {
        match self {
            Self::Compound(x) => x.size(),
            Self::Opaque(x) => x.size(),
            Self::Enum(x) => x.size(),
            Self::Vlen(x) => x.size(),
            Self::Int(x) => x.size(),
            Self::Float(x) => x.size(),
            Self::String => std::mem::size_of::<*const std::ffi::c_char>(),
            Self::Char => 1,
        }
    }
}

/// Opaque blob of bytes with a name
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OpaqueType {
    /// Name of type
    pub name: String,
    /// Size of type in bytes
    pub size: usize,
}
impl OpaqueType {
    fn size(&self) -> usize {
        self.size
    }
}

/// Integer type used in `netCDF`
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[allow(missing_docs)]
pub enum IntType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
}
impl IntType {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn size(&self) -> usize {
        match self {
            Self::U8 | Self::I8 => 1,
            Self::U16 | Self::I16 => 2,
            Self::U32 | Self::I32 => 4,
            Self::U64 | Self::I64 => 8,
        }
    }
}

/// Floating type used in `netCDF`
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[allow(missing_docs)]
pub enum FloatType {
    F32,
    F64,
}
impl FloatType {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn size(&self) -> usize {
        match self {
            Self::F32 => 4,
            Self::F64 => 8,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
/// Field of a compound struct
pub struct CompoundTypeField {
    /// Name of the compound field
    pub name: String,
    /// Type of the compound field
    pub basetype: NcVariableType,
    /// Dimensionality of the compound field (if any)
    pub arraydims: Option<Vec<usize>>,
    /// Offset of this field (in bytes) relative to the start
    /// of the compound
    pub offset: usize,
}

#[derive(Clone, Debug)]
/// Compound/record type
pub struct CompoundType {
    /// Name of the compound
    pub name: String,
    /// Size in bytes of the compound
    pub size: usize,
    /// Fields of the compound
    pub fields: Vec<CompoundTypeField>,
}
impl CompoundType {
    fn size(&self) -> usize {
        self.size
    }
}

impl PartialEq for CompoundType {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }
        if self.fields.len() != other.fields.len() {
            return false;
        }
        if self.size() != other.size() {
            return false;
        }
        if self.fields.is_empty() {
            return true;
        }
        if self.fields == other.fields {
            return true;
        }
        // Check if fields are equal if ordered differently
        // by checking each element against the other,
        // done both ways to ensure each element has a match
        if !self
            .fields
            .iter()
            .all(|x| other.fields.iter().any(|y| x == y))
        {
            return false;
        }
        other
            .fields
            .iter()
            .all(|x| self.fields.iter().any(|y| x == y))
    }
}

impl Eq for CompoundType {}

#[derive(Clone, PartialEq, Eq, Debug)]
/// Inner values of the enum type
///
/// `netCDF` only supports integer types
#[allow(missing_docs)]
pub enum EnumTypeValues {
    U8(Vec<u8>),
    U16(Vec<u16>),
    U32(Vec<u32>),
    U64(Vec<u64>),
    I8(Vec<i8>),
    I16(Vec<i16>),
    I32(Vec<i32>),
    I64(Vec<i64>),
}
impl EnumTypeValues {
    /// Inner type of the enum
    fn nc_type(&self) -> netcdf_sys::nc_type {
        use netcdf_sys::*;
        match self {
            Self::U8(_) => NC_UBYTE,
            Self::I8(_) => NC_BYTE,
            Self::U16(_) => NC_USHORT,
            Self::I16(_) => NC_SHORT,
            Self::U32(_) => NC_UINT,
            Self::I32(_) => NC_INT,
            Self::U64(_) => NC_UINT64,
            Self::I64(_) => NC_INT64,
        }
    }
}
macro_rules! from_vec {
    ($ty: ty, $item: expr) => {
        impl From<Vec<$ty>> for EnumTypeValues {
            fn from(v: Vec<$ty>) -> Self {
                $item(v)
            }
        }
    };
}
from_vec!(u8, Self::U8);
from_vec!(u16, Self::U16);
from_vec!(u32, Self::U32);
from_vec!(u64, Self::U64);
from_vec!(i8, Self::I8);
from_vec!(i16, Self::I16);
from_vec!(i32, Self::I32);
from_vec!(i64, Self::I64);

#[derive(Clone, Debug)]
/// Enum type
pub struct EnumType {
    /// Name of enum
    pub name: String,
    /// Name of enumeration fields
    pub fieldnames: Vec<String>,
    /// Values of enumeration fields
    pub fieldvalues: EnumTypeValues,
}
impl EnumType {
    /// Size of enum in bytes
    fn size(&self) -> usize {
        match self.fieldvalues {
            EnumTypeValues::U8(_) | EnumTypeValues::I8(_) => 1,
            EnumTypeValues::U16(_) | EnumTypeValues::I16(_) => 2,
            EnumTypeValues::U32(_) | EnumTypeValues::I32(_) => 4,
            EnumTypeValues::U64(_) | EnumTypeValues::I64(_) => 8,
        }
    }
}

impl PartialEq for EnumType {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }
        if self.fieldnames.len() != other.fieldnames.len() {
            return false;
        }
        if self.fieldnames.is_empty() {
            return true;
        }
        if self.fieldnames == other.fieldnames && self.fieldvalues == other.fieldvalues {
            return true;
        }

        // Check for enum fields ordered differently
        macro_rules! enumtype {
            ($x: expr, $y: expr) => {{
                if !self.fieldnames.iter().zip($x).all(|(x, sname)| {
                    other
                        .fieldnames
                        .iter()
                        .zip($y)
                        .any(|(y, oname)| sname == oname && x == y)
                }) {
                    return false;
                }
                other.fieldnames.iter().zip($y).all(|(y, oname)| {
                    self.fieldnames
                        .iter()
                        .zip($x)
                        .any(|(x, sname)| sname == oname && x == y)
                })
            }};
        }
        match (&self.fieldvalues, &other.fieldvalues) {
            (EnumTypeValues::U8(x), EnumTypeValues::U8(y)) => enumtype!(x, y),
            (EnumTypeValues::U16(x), EnumTypeValues::U16(y)) => enumtype!(x, y),
            (EnumTypeValues::U32(x), EnumTypeValues::U32(y)) => enumtype!(x, y),
            (EnumTypeValues::U64(x), EnumTypeValues::U64(y)) => enumtype!(x, y),
            (EnumTypeValues::I8(x), EnumTypeValues::I8(y)) => enumtype!(x, y),
            (EnumTypeValues::I16(x), EnumTypeValues::I16(y)) => enumtype!(x, y),
            (EnumTypeValues::I32(x), EnumTypeValues::I32(y)) => enumtype!(x, y),
            (EnumTypeValues::I64(x), EnumTypeValues::I64(y)) => enumtype!(x, y),
            _ => false,
        }
    }
}

impl Eq for EnumType {}

#[derive(Clone, PartialEq, Eq, Debug)]
/// Ragged array
pub struct VlenType {
    /// Name of type
    pub name: String,
    /// Inner type of array
    pub basetype: Box<NcVariableType>,
}
impl VlenType {
    #[allow(clippy::unused_self)]
    /// Size in bytes
    fn size(&self) -> usize {
        std::mem::size_of::<netcdf_sys::nc_vlen_t>()
    }
}

macro_rules! impl_basic {
    ($ty: ty, $item: expr) => {
        unsafe impl NcTypeDescriptor for $ty {
            fn type_descriptor() -> NcVariableType {
                $item
            }
        }
    };
}

#[rustfmt::skip]
impl_basic!(u8, NcVariableType::Int(IntType::U8));
#[rustfmt::skip]
impl_basic!(u16, NcVariableType::Int(IntType::U16));
#[rustfmt::skip]
impl_basic!(u32, NcVariableType::Int(IntType::U32));
#[rustfmt::skip]
impl_basic!(u64, NcVariableType::Int(IntType::U64));
#[rustfmt::skip]
impl_basic!(i8, NcVariableType::Int(IntType::I8));
#[rustfmt::skip]
impl_basic!(i16, NcVariableType::Int(IntType::I16));
#[rustfmt::skip]
impl_basic!(i32, NcVariableType::Int(IntType::I32));
#[rustfmt::skip]
impl_basic!(i64, NcVariableType::Int(IntType::I64));
#[rustfmt::skip]
impl_basic!(f32, NcVariableType::Float(FloatType::F32));
#[rustfmt::skip]
impl_basic!(f64, NcVariableType::Float(FloatType::F64));

#[doc(hidden)]
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug)]
pub enum ArrayElements {
    None,
    One([usize; 1]),
    Two([usize; 2]),
    Three([usize; 3]),
}

impl ArrayElements {
    pub fn as_dims(&self) -> Option<&[usize]> {
        match self {
            Self::None => None,
            Self::One(x) => Some(x.as_slice()),
            Self::Two(x) => Some(x.as_slice()),
            Self::Three(x) => Some(x.as_slice()),
        }
    }
}

macro_rules! impl_arrayed {
    ($typ: ty) => {
        #[doc(hidden)]
        unsafe impl<const N: usize> NcTypeDescriptor for [$typ; N] {
            fn type_descriptor() -> NcVariableType {
                <$typ as NcTypeDescriptor>::type_descriptor()
            }
            const ARRAY_ELEMENTS: ArrayElements = ArrayElements::One([N]);
        }
        #[doc(hidden)]
        unsafe impl<const N: usize, const M: usize> NcTypeDescriptor for [[$typ; N]; M] {
            fn type_descriptor() -> NcVariableType {
                <$typ as NcTypeDescriptor>::type_descriptor()
            }
            const ARRAY_ELEMENTS: ArrayElements = ArrayElements::Two([N, M]);
        }
        #[doc(hidden)]
        unsafe impl<const N: usize, const M: usize, const L: usize> NcTypeDescriptor
            for [[[$typ; N]; M]; L]
        {
            fn type_descriptor() -> NcVariableType {
                <$typ as NcTypeDescriptor>::type_descriptor()
            }
            const ARRAY_ELEMENTS: ArrayElements = ArrayElements::Three([N, M, L]);
        }
    };
}

impl_arrayed!(u8);
impl_arrayed!(u16);
impl_arrayed!(u32);
impl_arrayed!(u64);
impl_arrayed!(i8);
impl_arrayed!(i16);
impl_arrayed!(i32);
impl_arrayed!(i64);
impl_arrayed!(f32);
impl_arrayed!(f64);

/// Find a type from a given descriptor
pub(crate) fn find_type(ncid: nc_type, typ: &NcVariableType) -> Result<Option<nc_type>> {
    match *typ {
        NcVariableType::Int(IntType::U8) => return Ok(Some(NC_UBYTE)),
        NcVariableType::Int(IntType::I8) => return Ok(Some(NC_BYTE)),
        NcVariableType::Int(IntType::U16) => return Ok(Some(NC_USHORT)),
        NcVariableType::Int(IntType::I16) => return Ok(Some(NC_SHORT)),
        NcVariableType::Int(IntType::U32) => return Ok(Some(NC_UINT)),
        NcVariableType::Int(IntType::I32) => return Ok(Some(NC_INT)),
        NcVariableType::Int(IntType::U64) => return Ok(Some(NC_UINT64)),
        NcVariableType::Int(IntType::I64) => return Ok(Some(NC_INT64)),
        NcVariableType::Float(FloatType::F32) => return Ok(Some(NC_FLOAT)),
        NcVariableType::Float(FloatType::F64) => return Ok(Some(NC_DOUBLE)),
        NcVariableType::String => return Ok(Some(NC_STRING)),
        NcVariableType::Char => return Ok(Some(NC_CHAR)),
        _ => {}
    }

    let name = match &typ {
        NcVariableType::Compound(x) => &x.name,
        NcVariableType::Opaque(x) => &x.name,
        NcVariableType::Enum(x) => &x.name,
        NcVariableType::Vlen(x) => &x.name,
        _ => unreachable!(),
    };

    let mut typid = 0;
    let name = crate::utils::short_name_to_bytes(name)?;
    let e = checked_with_lock(|| unsafe {
        netcdf_sys::nc_inq_typeid(ncid, name.as_ptr().cast(), &mut typid)
    });
    if matches!(e, Err(crate::Error::Netcdf(netcdf_sys::NC_EBADTYPE))) {
        return Ok(None);
    }
    let candidate = read_type(ncid, typid)?;
    if &candidate != typ {
        return Err("Found type with that name, but it was not the correct type definition".into());
    }
    Ok(Some(typid))
}

/// Add a type from the given descriptor
pub(crate) fn add_type(ncid: nc_type, typ: NcVariableType, recursive: bool) -> Result<nc_type> {
    match typ {
        NcVariableType::Int(_)
        | NcVariableType::Float(_)
        | NcVariableType::String
        | NcVariableType::Char => Err("basic type can not be added".into()),
        NcVariableType::Opaque(x) => {
            let name = crate::utils::short_name_to_bytes(&x.name)?;
            let mut id = 0;
            checked_with_lock(|| unsafe {
                netcdf_sys::nc_def_opaque(ncid, x.size, name.as_ptr().cast(), &mut id)
            })?;
            Ok(id)
        }
        NcVariableType::Vlen(x) => {
            let mut id = 0;
            let name = crate::utils::short_name_to_bytes(&x.name)?;

            let othertype = find_type(ncid, &x.basetype)?;
            let othertype = if let Some(x) = othertype {
                x
            } else if recursive {
                add_type(ncid, *(x.basetype), recursive)?
            } else {
                return Err("Type not found".into());
            };
            // let basetype = find_type(ncid, &*x.name)?.expect("No type found");
            // let othertyp = read_type(ncid, x.basetype)?;

            checked_with_lock(|| unsafe {
                netcdf_sys::nc_def_vlen(ncid, name.as_ptr().cast(), othertype, &mut id)
            })?;
            Ok(id)
        }
        NcVariableType::Enum(EnumType {
            name,
            fieldnames,
            fieldvalues,
        }) => {
            let mut id = 0;
            let name = crate::utils::short_name_to_bytes(&name)?;
            let basetyp = fieldvalues.nc_type();
            checked_with_lock(|| unsafe {
                netcdf_sys::nc_def_enum(ncid, basetyp, name.as_ptr().cast(), &mut id)
            })?;

            macro_rules! write_fieldvalues {
                ($ty: ty, $x: expr) => {{
                    for (name, value) in fieldnames.iter().zip(&$x) {
                        let name = crate::utils::short_name_to_bytes(&name)?;
                        checked_with_lock(|| unsafe {
                            netcdf_sys::nc_insert_enum(
                                ncid,
                                id,
                                name.as_ptr().cast(),
                                (value as *const $ty).cast(),
                            )
                        })?;
                    }
                }};
            }

            match fieldvalues {
                EnumTypeValues::U8(x) => write_fieldvalues!(u8, x),
                EnumTypeValues::I8(x) => write_fieldvalues!(i8, x),
                EnumTypeValues::U16(x) => write_fieldvalues!(u16, x),
                EnumTypeValues::I16(x) => write_fieldvalues!(i16, x),
                EnumTypeValues::U32(x) => write_fieldvalues!(u32, x),
                EnumTypeValues::I32(x) => write_fieldvalues!(i32, x),
                EnumTypeValues::U64(x) => write_fieldvalues!(u64, x),
                EnumTypeValues::I64(x) => write_fieldvalues!(i64, x),
            }

            Ok(id)
        }
        NcVariableType::Compound(x) => {
            let mut xtypes = Vec::with_capacity(x.fields.len());
            for f in &x.fields {
                let xtype = find_type(ncid, &f.basetype)?;
                let xtype = match (recursive, xtype) {
                    (_, Some(xtype)) => xtype,
                    (true, None) => add_type(ncid, f.basetype.clone(), recursive)?,
                    (false, None) => return Err("Could not find subtype".into()),
                };
                xtypes.push(xtype);
            }
            let mut id = 0;
            let name = crate::utils::short_name_to_bytes(&x.name)?;
            checked_with_lock(|| unsafe {
                netcdf_sys::nc_def_compound(ncid, x.size(), name.as_ptr().cast(), &mut id)
            })?;

            // Find all subtypes, check if compatible, add if necessary
            for (f, xtype) in x.fields.iter().zip(xtypes) {
                let fieldname = crate::utils::short_name_to_bytes(&f.name)?;
                match f.arraydims {
                    None => checked_with_lock(|| unsafe {
                        netcdf_sys::nc_insert_compound(
                            ncid,
                            id,
                            fieldname.as_ptr().cast(),
                            f.offset,
                            xtype,
                        )
                    })?,
                    Some(ref x) => {
                        let ndims = x.len() as _;
                        let dims = x.iter().map(|&x| x as _).collect::<Vec<_>>();
                        checked_with_lock(|| unsafe {
                            netcdf_sys::nc_insert_array_compound(
                                ncid,
                                id,
                                fieldname.as_ptr().cast(),
                                f.offset,
                                xtype,
                                ndims,
                                dims.as_ptr(),
                            )
                        })?
                    }
                }
            }
            Ok(id)
        }
    }
}

#[allow(clippy::too_many_lines)]
/// Read a type and return the descriptor belonging to the id of the type
pub(crate) fn read_type(ncid: nc_type, xtype: nc_type) -> Result<NcVariableType> {
    match xtype {
        NC_UBYTE => return Ok(NcVariableType::Int(IntType::U8)),
        NC_BYTE => return Ok(NcVariableType::Int(IntType::I8)),
        NC_USHORT => return Ok(NcVariableType::Int(IntType::U16)),
        NC_SHORT => return Ok(NcVariableType::Int(IntType::I16)),
        NC_UINT => return Ok(NcVariableType::Int(IntType::U32)),
        NC_INT => return Ok(NcVariableType::Int(IntType::I32)),
        NC_UINT64 => return Ok(NcVariableType::Int(IntType::U64)),
        NC_INT64 => return Ok(NcVariableType::Int(IntType::I64)),
        NC_FLOAT => return Ok(NcVariableType::Float(FloatType::F32)),
        NC_DOUBLE => return Ok(NcVariableType::Float(FloatType::F64)),
        NC_STRING => return Ok(NcVariableType::String),
        NC_CHAR => return Ok(NcVariableType::Char),
        _ => {}
    }
    let mut base_xtype = 0;
    let mut name = [0_u8; NC_MAX_NAME as usize + 1];
    let mut size = 0;
    let mut base_enum_type = 0;
    let mut fieldmembers = 0;
    checked_with_lock(|| unsafe {
        netcdf_sys::nc_inq_user_type(
            ncid,
            xtype,
            name.as_mut_ptr().cast(),
            &mut size,
            &mut base_enum_type,
            &mut fieldmembers,
            &mut base_xtype,
        )
    })?;
    let name = std::ffi::CStr::from_bytes_until_nul(name.as_ref())
        .unwrap()
        .to_str()
        .unwrap();
    match base_xtype {
        NC_VLEN => {
            let basetype = read_type(ncid, base_enum_type)?;
            Ok(NcVariableType::Vlen(VlenType {
                name: name.to_owned(),
                basetype: Box::new(basetype),
            }))
        }
        NC_OPAQUE => Ok(NcVariableType::Opaque(OpaqueType {
            name: name.to_owned(),
            size,
        })),
        NC_ENUM => {
            let mut fieldnames = vec![];
            for idx in 0..fieldmembers {
                let mut cname = [0_u8; NC_MAX_NAME as usize + 1];
                let idx = idx.try_into().unwrap();
                checked_with_lock(|| unsafe {
                    netcdf_sys::nc_inq_enum_member(
                        ncid,
                        xtype,
                        idx,
                        cname.as_mut_ptr().cast(),
                        std::ptr::null_mut(),
                    )
                })?;
                let cstr = std::ffi::CStr::from_bytes_until_nul(cname.as_slice()).unwrap();
                fieldnames.push(cstr.to_str().unwrap().to_owned());
            }
            macro_rules! read_fieldvalues {
                ($ty: ty) => {{
                    let mut values = vec![0 as $ty; fieldmembers];
                    for (idx, value) in values.iter_mut().enumerate() {
                        checked_with_lock(|| unsafe {
                            netcdf_sys::nc_inq_enum_member(
                                ncid,
                                xtype,
                                idx as _,
                                std::ptr::null_mut(),
                                (value as *mut $ty).cast(),
                            )
                        })?;
                    }
                    values.into()
                }};
            }
            let fieldvalues = match base_enum_type {
                NC_BYTE => {
                    read_fieldvalues!(i8)
                }
                NC_UBYTE => {
                    read_fieldvalues!(u8)
                }
                NC_SHORT => {
                    read_fieldvalues!(i16)
                }
                NC_USHORT => {
                    read_fieldvalues!(u16)
                }
                NC_INT => {
                    read_fieldvalues!(i32)
                }
                NC_UINT => {
                    read_fieldvalues!(u32)
                }
                NC_INT64 => {
                    read_fieldvalues!(i64)
                }
                NC_UINT64 => {
                    read_fieldvalues!(u64)
                }
                _ => unreachable!("netCDF does not support {base_enum_type} as type in enum"),
            };
            Ok(NcVariableType::Enum(EnumType {
                name: name.to_owned(),
                fieldnames,
                fieldvalues,
            }))
        }
        NC_COMPOUND => {
            let mut fields = vec![];
            for fieldid in 0..fieldmembers {
                let mut fieldname = [0; NC_MAX_NAME as usize + 1];
                let mut fieldtype = 0;
                let mut ndims = 0;
                let mut arraydims = None;
                let mut offset = 0;
                let fieldid = fieldid.try_into().unwrap();
                checked_with_lock(|| unsafe {
                    netcdf_sys::nc_inq_compound_field(
                        ncid,
                        xtype,
                        fieldid,
                        fieldname.as_mut_ptr().cast(),
                        &mut offset,
                        &mut fieldtype,
                        &mut ndims,
                        std::ptr::null_mut(),
                    )
                })?;
                if ndims != 0 {
                    let mut dimsizes = vec![0; ndims.try_into().unwrap()];
                    checked_with_lock(|| unsafe {
                        netcdf_sys::nc_inq_compound_fielddim_sizes(
                            ncid,
                            xtype,
                            fieldid,
                            dimsizes.as_mut_ptr(),
                        )
                    })?;
                    arraydims = Some(dimsizes.iter().map(|&x| x.try_into().unwrap()).collect());
                }
                let fieldname = std::ffi::CStr::from_bytes_until_nul(fieldname.as_slice()).unwrap();

                fields.push(CompoundTypeField {
                    name: fieldname.to_str().unwrap().to_owned(),
                    basetype: read_type(ncid, fieldtype)?,
                    arraydims,
                    offset,
                });
            }
            Ok(NcVariableType::Compound(CompoundType {
                name: name.to_owned(),
                size,
                fields,
            }))
        }
        _ => panic!("Unexcepted base type {base_xtype}"),
    }
}

/// Find all user-defined types at the location
pub(crate) fn all_at_location(
    ncid: nc_type,
) -> Result<impl Iterator<Item = Result<NcVariableType>>> {
    let typeids = {
        let mut num_typeids = 0;
        checked_with_lock(|| unsafe {
            netcdf_sys::nc_inq_typeids(ncid, &mut num_typeids, std::ptr::null_mut())
        })?;
        let mut typeids = vec![0; num_typeids.try_into()?];
        checked_with_lock(|| unsafe {
            netcdf_sys::nc_inq_typeids(ncid, std::ptr::null_mut(), typeids.as_mut_ptr())
        })?;
        typeids
    };
    Ok(typeids.into_iter().map(move |x| read_type(ncid, x)))
}

#[repr(transparent)]
/// `NC_STRING` compatible struct, no drop implementation, use with caution
pub(crate) struct NcString(pub(crate) *mut std::ffi::c_char);
unsafe impl NcTypeDescriptor for NcString {
    fn type_descriptor() -> NcVariableType {
        NcVariableType::String
    }
}
