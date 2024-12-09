//! Add and read attributes from netcdf groups and variables
#![allow(clippy::similar_names)]

use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::c_char;

use netcdf_sys::*;

use super::error;
use super::utils::{checked_with_lock, with_lock};

/// Extra properties of a variable or a group can be represented
/// with attributes. Primarily added with `add_attribute` on
/// the variable and group
#[derive(Clone)]
pub struct Attribute<'a> {
    pub(crate) name: [u8; NC_MAX_NAME as usize + 1],
    /// Group or file this attribute is in
    pub(crate) ncid: nc_type,
    /// Variable/global this id is connected to. This is
    /// set to NC_GLOBAL when attached to a group
    pub(crate) varid: nc_type,
    /// Holds the variable/group to prevent the
    /// attribute being deleted or modified from
    /// under us
    pub(crate) _marker: PhantomData<&'a nc_type>,
}

impl std::fmt::Debug for Attribute<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: {}", self.name())?;
        write!(f, "ncid: {}", self.ncid)?;
        write!(f, "varid: {}", self.varid)
    }
}

impl Attribute<'_> {
    /// Get the name of the attribute
    ///
    /// # Panics
    /// attribute could have a name containing an invalid utf8-sequence
    pub fn name(&self) -> &str {
        let zeropos = self
            .name
            .iter()
            .position(|&x| x == 0)
            .unwrap_or(self.name.len());
        std::str::from_utf8(&self.name[..zeropos])
            .expect("Attribute name contains invalid sequence")
    }
    /// Number of elements in this attribute
    fn num_elems(&self) -> error::Result<usize> {
        let mut nelems = 0;
        checked_with_lock(|| unsafe {
            nc_inq_attlen(
                self.ncid,
                self.varid,
                self.name.as_ptr().cast(),
                &mut nelems,
            )
        })?;
        Ok(nelems as _)
    }
    /// Type of this attribute
    fn typ(&self) -> error::Result<nc_type> {
        let mut atttype = 0;
        checked_with_lock(|| unsafe {
            nc_inq_atttype(
                self.ncid,
                self.varid,
                self.name.as_ptr().cast(),
                &mut atttype,
            )
        })?;

        Ok(atttype)
    }
    /// Get the value of the attribute
    ///
    /// # Errors
    ///
    /// Unsupported type or netcdf error
    #[allow(clippy::too_many_lines)]
    pub fn value(&self) -> error::Result<AttributeValue> {
        let attlen = self.num_elems()?;
        let typ = self.typ()?;

        match typ {
            NC_UBYTE => match attlen {
                1 => {
                    let mut value = 0;
                    checked_with_lock(|| unsafe {
                        nc_get_att_uchar(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            &mut value,
                        )
                    })?;
                    Ok(AttributeValue::Uchar(value))
                }
                len => {
                    let mut values = vec![0_u8; len];
                    checked_with_lock(|| unsafe {
                        nc_get_att_uchar(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            values.as_mut_ptr(),
                        )
                    })?;
                    Ok(AttributeValue::Uchars(values))
                }
            },
            NC_BYTE => match attlen {
                1 => {
                    let mut value = 0;
                    checked_with_lock(|| unsafe {
                        nc_get_att_schar(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            &mut value,
                        )
                    })?;

                    Ok(AttributeValue::Schar(value))
                }
                len => {
                    let mut values = vec![0; len as _];
                    checked_with_lock(|| unsafe {
                        nc_get_att_schar(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            values.as_mut_ptr(),
                        )
                    })?;
                    Ok(AttributeValue::Schars(values))
                }
            },
            NC_SHORT => match attlen {
                1 => {
                    let mut value = 0;
                    checked_with_lock(|| unsafe {
                        nc_get_att_short(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            &mut value,
                        )
                    })?;
                    Ok(AttributeValue::Short(value))
                }
                len => {
                    let mut values = vec![0; len as _];
                    checked_with_lock(|| unsafe {
                        nc_get_att_short(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            values.as_mut_ptr(),
                        )
                    })?;
                    Ok(AttributeValue::Shorts(values))
                }
            },
            NC_USHORT => match attlen {
                1 => {
                    let mut value = 0;
                    checked_with_lock(|| unsafe {
                        nc_get_att_ushort(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            &mut value,
                        )
                    })?;
                    Ok(AttributeValue::Ushort(value))
                }
                len => {
                    let mut values = vec![0; len as _];
                    checked_with_lock(|| unsafe {
                        nc_get_att_ushort(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            values.as_mut_ptr(),
                        )
                    })?;
                    Ok(AttributeValue::Ushorts(values))
                }
            },
            NC_INT => match attlen {
                1 => {
                    let mut value = 0;
                    checked_with_lock(|| unsafe {
                        nc_get_att_int(self.ncid, self.varid, self.name.as_ptr().cast(), &mut value)
                    })?;
                    Ok(AttributeValue::Int(value))
                }
                len => {
                    let mut values = vec![0; len as _];
                    checked_with_lock(|| unsafe {
                        nc_get_att_int(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            values.as_mut_ptr(),
                        )
                    })?;
                    Ok(AttributeValue::Ints(values))
                }
            },
            NC_UINT => match attlen {
                1 => {
                    let mut value = 0;
                    checked_with_lock(|| unsafe {
                        nc_get_att_uint(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            &mut value,
                        )
                    })?;
                    Ok(AttributeValue::Uint(value))
                }
                len => {
                    let mut values = vec![0; len as _];
                    checked_with_lock(|| unsafe {
                        nc_get_att_uint(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            values.as_mut_ptr(),
                        )
                    })?;
                    Ok(AttributeValue::Uints(values))
                }
            },
            NC_INT64 => match attlen {
                1 => {
                    let mut value = 0;
                    checked_with_lock(|| unsafe {
                        nc_get_att_longlong(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            &mut value,
                        )
                    })?;
                    Ok(AttributeValue::Longlong(value))
                }
                len => {
                    let mut values = vec![0; len as _];
                    checked_with_lock(|| unsafe {
                        nc_get_att_longlong(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            values.as_mut_ptr(),
                        )
                    })?;

                    Ok(AttributeValue::Longlongs(values))
                }
            },
            NC_UINT64 => match attlen {
                1 => {
                    let mut value = 0;
                    checked_with_lock(|| unsafe {
                        nc_get_att_ulonglong(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            &mut value,
                        )
                    })?;
                    Ok(AttributeValue::Ulonglong(value))
                }
                len => {
                    let mut values = vec![0; len as _];
                    checked_with_lock(|| unsafe {
                        nc_get_att_ulonglong(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            values.as_mut_ptr(),
                        )
                    })?;

                    Ok(AttributeValue::Ulonglongs(values))
                }
            },
            NC_FLOAT => match attlen {
                1 => {
                    let mut value = 0.0;
                    checked_with_lock(|| unsafe {
                        nc_get_att_float(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            &mut value,
                        )
                    })?;
                    Ok(AttributeValue::Float(value))
                }
                len => {
                    let mut values = vec![0.0; len as _];
                    checked_with_lock(|| unsafe {
                        nc_get_att_float(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            values.as_mut_ptr(),
                        )
                    })?;
                    Ok(AttributeValue::Floats(values))
                }
            },
            NC_DOUBLE => match attlen {
                1 => {
                    let mut value = 0.0;
                    checked_with_lock(|| unsafe {
                        nc_get_att_double(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            &mut value,
                        )
                    })?;
                    Ok(AttributeValue::Double(value))
                }
                len => {
                    let mut values = vec![0.0; len as _];
                    checked_with_lock(|| unsafe {
                        nc_get_att_double(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr().cast(),
                            values.as_mut_ptr(),
                        )
                    })?;
                    Ok(AttributeValue::Doubles(values))
                }
            },
            NC_CHAR => {
                let lentext = attlen;
                let mut buf: Vec<u8> = vec![0; lentext as _];
                checked_with_lock(|| unsafe {
                    nc_get_att_text(
                        self.ncid,
                        self.varid,
                        self.name.as_ptr().cast(),
                        buf.as_mut_ptr().cast::<u8>().cast::<c_char>(),
                    )
                })?;
                let pos = buf.iter().position(|&x| x == 0).unwrap_or(buf.len());
                Ok(AttributeValue::Str(String::from(String::from_utf8_lossy(
                    &buf[..pos],
                ))))
            }
            NC_STRING => {
                let mut buf: Vec<*mut c_char> = vec![std::ptr::null_mut(); attlen];
                checked_with_lock(|| unsafe {
                    nc_get_att_string(
                        self.ncid,
                        self.varid,
                        self.name.as_ptr().cast(),
                        buf.as_mut_ptr().cast(),
                    )
                })?;
                let result = buf
                    .iter()
                    .map(|cstr_pointer| unsafe {
                        if cstr_pointer.is_null() {
                            String::new()
                        } else {
                            CStr::from_ptr(*cstr_pointer).to_string_lossy().to_string()
                        }
                    })
                    .collect();
                with_lock(|| unsafe { nc_free_string(attlen, buf.as_mut_ptr()) });
                Ok(AttributeValue::Strs(result))
            }
            x => Err(error::Error::TypeUnknown(x)),
        }
    }
}

/// Iterator over all attributes for a location
pub(crate) struct AttributeIterator<'a> {
    ncid: nc_type,
    varid: Option<nc_type>,
    natts: usize,
    current_natt: usize,
    _marker: PhantomData<&'a nc_type>,
}

impl AttributeIterator<'_> {
    pub(crate) fn new(ncid: nc_type, varid: Option<nc_type>) -> error::Result<Self> {
        let mut natts = 0;
        checked_with_lock(|| unsafe {
            nc_inq_varnatts(ncid, varid.unwrap_or(NC_GLOBAL), &mut natts)
        })?;
        Ok(Self {
            ncid,
            varid,
            natts: natts.try_into()?,
            current_natt: 0,
            _marker: PhantomData,
        })
    }
}

impl<'a> Iterator for AttributeIterator<'a> {
    type Item = error::Result<Attribute<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_natt >= self.natts {
            return None;
        }

        let mut name = [0_u8; NC_MAX_NAME as usize + 1];
        if let Err(e) = checked_with_lock(|| unsafe {
            nc_inq_attname(
                self.ncid,
                self.varid.unwrap_or(NC_GLOBAL),
                self.current_natt.try_into().unwrap(),
                name.as_mut_ptr().cast(),
            )
        }) {
            return Some(Err(e));
        }

        let att = Attribute {
            name,
            ncid: self.ncid,
            varid: self.varid.unwrap_or(NC_GLOBAL),
            _marker: PhantomData,
        };

        self.current_natt += 1;
        Some(Ok(att))
    }
}

/// Holds the attribute value which can be inserted and
/// returned from the file
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    Uchar(u8),
    Uchars(Vec<u8>),
    Schar(i8),
    Schars(Vec<i8>),
    Ushort(u16),
    Ushorts(Vec<u16>),
    Short(i16),
    Shorts(Vec<i16>),
    Uint(u32),
    Uints(Vec<u32>),
    Int(i32),
    Ints(Vec<i32>),
    Ulonglong(u64),
    Ulonglongs(Vec<u64>),
    Longlong(i64),
    Longlongs(Vec<i64>),
    Float(f32),
    Floats(Vec<f32>),
    Double(f64),
    Doubles(Vec<f64>),
    Str(String),
    Strs(Vec<String>),
}

impl Attribute<'_> {
    #[allow(clippy::needless_pass_by_value)] // All values will be small
    #[allow(clippy::too_many_lines)]
    pub(crate) fn put(
        ncid: nc_type,
        varid: nc_type,
        name: &str,
        val: AttributeValue,
    ) -> error::Result<Self> {
        let cname = super::utils::short_name_to_bytes(name)?;

        match val {
            AttributeValue::Uchar(x) => checked_with_lock(|| unsafe {
                nc_put_att_uchar(ncid, varid, cname.as_ptr().cast(), NC_UBYTE, 1, &x)
            }),
            AttributeValue::Uchars(x) => checked_with_lock(|| unsafe {
                nc_put_att_uchar(
                    ncid,
                    varid,
                    cname.as_ptr().cast(),
                    NC_UBYTE,
                    x.len(),
                    x.as_ptr(),
                )
            }),
            AttributeValue::Schar(x) => checked_with_lock(|| unsafe {
                nc_put_att_schar(ncid, varid, cname.as_ptr().cast(), NC_BYTE, 1, &x)
            }),
            AttributeValue::Schars(x) => checked_with_lock(|| unsafe {
                nc_put_att_schar(
                    ncid,
                    varid,
                    cname.as_ptr().cast(),
                    NC_BYTE,
                    x.len(),
                    x.as_ptr(),
                )
            }),
            AttributeValue::Ushort(x) => checked_with_lock(|| unsafe {
                nc_put_att_ushort(ncid, varid, cname.as_ptr().cast(), NC_USHORT, 1, &x)
            }),
            AttributeValue::Ushorts(x) => checked_with_lock(|| unsafe {
                nc_put_att_ushort(
                    ncid,
                    varid,
                    cname.as_ptr().cast(),
                    NC_USHORT,
                    x.len(),
                    x.as_ptr(),
                )
            }),
            AttributeValue::Short(x) => checked_with_lock(|| unsafe {
                nc_put_att_short(ncid, varid, cname.as_ptr().cast(), NC_SHORT, 1, &x)
            }),
            AttributeValue::Shorts(x) => checked_with_lock(|| unsafe {
                nc_put_att_short(
                    ncid,
                    varid,
                    cname.as_ptr().cast(),
                    NC_SHORT,
                    x.len(),
                    x.as_ptr(),
                )
            }),
            AttributeValue::Uint(x) => checked_with_lock(|| unsafe {
                nc_put_att_uint(ncid, varid, cname.as_ptr().cast(), NC_UINT, 1, &x)
            }),
            AttributeValue::Uints(x) => checked_with_lock(|| unsafe {
                nc_put_att_uint(
                    ncid,
                    varid,
                    cname.as_ptr().cast(),
                    NC_UINT,
                    x.len(),
                    x.as_ptr(),
                )
            }),
            AttributeValue::Int(x) => checked_with_lock(|| unsafe {
                nc_put_att_int(ncid, varid, cname.as_ptr().cast(), NC_INT, 1, &x)
            }),
            AttributeValue::Ints(x) => checked_with_lock(|| unsafe {
                nc_put_att_int(
                    ncid,
                    varid,
                    cname.as_ptr().cast(),
                    NC_INT,
                    x.len(),
                    x.as_ptr(),
                )
            }),
            AttributeValue::Ulonglong(x) => checked_with_lock(|| unsafe {
                nc_put_att_ulonglong(ncid, varid, cname.as_ptr().cast(), NC_UINT64, 1, &x)
            }),
            AttributeValue::Ulonglongs(x) => checked_with_lock(|| unsafe {
                nc_put_att_ulonglong(
                    ncid,
                    varid,
                    cname.as_ptr().cast(),
                    NC_UINT64,
                    x.len(),
                    x.as_ptr(),
                )
            }),
            AttributeValue::Longlong(x) => checked_with_lock(|| unsafe {
                nc_put_att_longlong(ncid, varid, cname.as_ptr().cast(), NC_INT64, 1, &x)
            }),
            AttributeValue::Longlongs(x) => checked_with_lock(|| unsafe {
                nc_put_att_longlong(
                    ncid,
                    varid,
                    cname.as_ptr().cast(),
                    NC_INT64,
                    x.len(),
                    x.as_ptr(),
                )
            }),
            AttributeValue::Float(x) => checked_with_lock(|| unsafe {
                nc_put_att_float(ncid, varid, cname.as_ptr().cast(), NC_FLOAT, 1, &x)
            }),
            AttributeValue::Floats(x) => checked_with_lock(|| unsafe {
                nc_put_att_float(
                    ncid,
                    varid,
                    cname.as_ptr().cast(),
                    NC_FLOAT,
                    x.len(),
                    x.as_ptr(),
                )
            }),
            AttributeValue::Double(x) => checked_with_lock(|| unsafe {
                nc_put_att_double(ncid, varid, cname.as_ptr().cast(), NC_DOUBLE, 1, &x)
            }),
            AttributeValue::Doubles(x) => checked_with_lock(|| unsafe {
                nc_put_att_double(
                    ncid,
                    varid,
                    cname.as_ptr().cast(),
                    NC_DOUBLE,
                    x.len(),
                    x.as_ptr(),
                )
            }),
            AttributeValue::Str(ref x) => checked_with_lock(|| unsafe {
                nc_put_att_text(
                    ncid,
                    varid,
                    cname.as_ptr().cast(),
                    x.len(),
                    x.as_ptr().cast(),
                )
            }),
            AttributeValue::Strs(ref x) => {
                let cstrings: Vec<CString> = x
                    .iter()
                    .map(String::as_str)
                    .map(CString::new)
                    .collect::<Result<Vec<CString>, _>>()?;

                let cstring_pointers: Vec<*const c_char> =
                    cstrings.iter().map(|cs| cs.as_ptr()).collect();

                checked_with_lock(|| unsafe {
                    nc_put_att_string(
                        ncid,
                        varid,
                        cname.as_ptr().cast(),
                        cstring_pointers.len(),
                        cstring_pointers.as_ptr().cast_mut(),
                    )
                })
            }
        }?;

        Ok(Self {
            name: cname,
            ncid,
            varid,
            _marker: PhantomData,
        })
    }

    pub(crate) fn find_from_name(
        ncid: nc_type,
        varid: Option<nc_type>,
        name: &str,
    ) -> error::Result<Option<Self>> {
        let attname = {
            if name.len() > NC_MAX_NAME as usize {
                return Err(error::Error::Netcdf(NC_EMAXNAME));
            }
            let mut attname = [0_u8; NC_MAX_NAME as usize + 1];
            attname[..name.len()].copy_from_slice(name.as_bytes());
            attname
        };
        let e = with_lock(|| unsafe {
            // Checking whether the variable exists by probing for its id
            nc_inq_attid(
                ncid,
                varid.unwrap_or(NC_GLOBAL),
                attname.as_ptr().cast(),
                std::ptr::null_mut(),
            )
        });
        if e == NC_ENOTATT {
            return Ok(None);
        }
        error::checked(e)?;

        Ok(Some(Attribute {
            name: attname,
            ncid,
            varid: varid.unwrap_or(NC_GLOBAL),
            _marker: PhantomData,
        }))
    }
}

// Boring implementations
impl From<u8> for AttributeValue {
    fn from(x: u8) -> Self {
        Self::Uchar(x)
    }
}
impl From<Vec<u8>> for AttributeValue {
    fn from(x: Vec<u8>) -> Self {
        Self::Uchars(x)
    }
}
impl From<i8> for AttributeValue {
    fn from(x: i8) -> Self {
        Self::Schar(x)
    }
}
impl From<Vec<i8>> for AttributeValue {
    fn from(x: Vec<i8>) -> Self {
        Self::Schars(x)
    }
}
impl From<u16> for AttributeValue {
    fn from(x: u16) -> Self {
        Self::Ushort(x)
    }
}
impl From<Vec<u16>> for AttributeValue {
    fn from(x: Vec<u16>) -> Self {
        Self::Ushorts(x)
    }
}
impl From<i16> for AttributeValue {
    fn from(x: i16) -> Self {
        Self::Short(x)
    }
}
impl From<Vec<i16>> for AttributeValue {
    fn from(x: Vec<i16>) -> Self {
        Self::Shorts(x)
    }
}
impl From<u32> for AttributeValue {
    fn from(x: u32) -> Self {
        Self::Uint(x)
    }
}
impl From<Vec<u32>> for AttributeValue {
    fn from(x: Vec<u32>) -> Self {
        Self::Uints(x)
    }
}
impl From<i32> for AttributeValue {
    fn from(x: i32) -> Self {
        Self::Int(x)
    }
}
impl From<Vec<i32>> for AttributeValue {
    fn from(x: Vec<i32>) -> Self {
        Self::Ints(x)
    }
}
impl From<u64> for AttributeValue {
    fn from(x: u64) -> Self {
        Self::Ulonglong(x)
    }
}
impl From<Vec<u64>> for AttributeValue {
    fn from(x: Vec<u64>) -> Self {
        Self::Ulonglongs(x)
    }
}
impl From<i64> for AttributeValue {
    fn from(x: i64) -> Self {
        Self::Longlong(x)
    }
}
impl From<Vec<i64>> for AttributeValue {
    fn from(x: Vec<i64>) -> Self {
        Self::Longlongs(x)
    }
}
impl From<f32> for AttributeValue {
    fn from(x: f32) -> Self {
        Self::Float(x)
    }
}
impl From<Vec<f32>> for AttributeValue {
    fn from(x: Vec<f32>) -> Self {
        Self::Floats(x)
    }
}
impl From<f64> for AttributeValue {
    fn from(x: f64) -> Self {
        Self::Double(x)
    }
}
impl From<Vec<f64>> for AttributeValue {
    fn from(x: Vec<f64>) -> Self {
        Self::Doubles(x)
    }
}
impl From<&str> for AttributeValue {
    fn from(x: &str) -> Self {
        Self::Str(x.to_string())
    }
}
impl From<String> for AttributeValue {
    fn from(x: String) -> Self {
        Self::Str(x)
    }
}
impl From<Vec<String>> for AttributeValue {
    fn from(x: Vec<String>) -> Self {
        Self::Strs(x)
    }
}
impl From<&[String]> for AttributeValue {
    fn from(x: &[String]) -> Self {
        Self::Strs(x.to_vec())
    }
}
impl From<&[&str]> for AttributeValue {
    fn from(x: &[&str]) -> Self {
        Self::Strs(x.iter().map(|&s| String::from(s)).collect())
    }
}
impl From<Vec<&str>> for AttributeValue {
    fn from(x: Vec<&str>) -> Self {
        Self::from(x.as_slice())
    }
}

#[test]
fn conversion() {
    let x = 1.0f32;
    let _b: AttributeValue = x.into();
}

impl TryFrom<AttributeValue> for u8 {
    type Error = error::Error;
    fn try_from(attr: AttributeValue) -> Result<Self, Self::Error> {
        match attr {
            AttributeValue::Uchar(x) => Ok(x),
            AttributeValue::Schar(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Ushort(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Short(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Uint(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Int(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Ulonglong(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Longlong(x) => (x).try_into().map_err(error::Error::Conversion),
            _ => Err("Conversion not supported".into()),
        }
    }
}

impl TryFrom<AttributeValue> for i8 {
    type Error = error::Error;
    fn try_from(attr: AttributeValue) -> Result<Self, Self::Error> {
        match attr {
            AttributeValue::Uchar(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Schar(x) => Ok(x),
            AttributeValue::Ushort(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Short(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Uint(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Int(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Ulonglong(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Longlong(x) => (x).try_into().map_err(error::Error::Conversion),
            _ => Err("Conversion not supported".into()),
        }
    }
}

impl TryFrom<AttributeValue> for u16 {
    type Error = error::Error;
    fn try_from(attr: AttributeValue) -> Result<Self, Self::Error> {
        match attr {
            AttributeValue::Uchar(x) => Ok((x).into()),
            AttributeValue::Schar(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Ushort(x) => Ok(x),
            AttributeValue::Short(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Uint(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Int(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Ulonglong(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Longlong(x) => (x).try_into().map_err(error::Error::Conversion),
            _ => Err("Conversion not supported".into()),
        }
    }
}

impl TryFrom<AttributeValue> for i16 {
    type Error = error::Error;
    fn try_from(attr: AttributeValue) -> Result<Self, Self::Error> {
        match attr {
            AttributeValue::Uchar(x) => Ok((x).into()),
            AttributeValue::Schar(x) => Ok((x).into()),
            AttributeValue::Ushort(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Short(x) => Ok(x),
            AttributeValue::Uint(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Int(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Ulonglong(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Longlong(x) => (x).try_into().map_err(error::Error::Conversion),
            _ => Err("Conversion not supported".into()),
        }
    }
}
impl TryFrom<AttributeValue> for u32 {
    type Error = error::Error;
    fn try_from(attr: AttributeValue) -> Result<Self, Self::Error> {
        match attr {
            AttributeValue::Uchar(x) => Ok((x).into()),
            AttributeValue::Schar(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Ushort(x) => Ok((x).into()),
            AttributeValue::Short(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Uint(x) => Ok(x),
            AttributeValue::Int(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Ulonglong(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Longlong(x) => (x).try_into().map_err(error::Error::Conversion),
            _ => Err("Conversion not supported".into()),
        }
    }
}
impl TryFrom<AttributeValue> for i32 {
    type Error = error::Error;
    fn try_from(attr: AttributeValue) -> Result<Self, Self::Error> {
        match attr {
            AttributeValue::Uchar(x) => Ok((x).into()),
            AttributeValue::Schar(x) => Ok((x).into()),
            AttributeValue::Ushort(x) => Ok((x).into()),
            AttributeValue::Short(x) => Ok((x).into()),
            AttributeValue::Uint(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Int(x) => Ok(x),
            AttributeValue::Ulonglong(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Longlong(x) => (x).try_into().map_err(error::Error::Conversion),
            _ => Err("Conversion not supported".into()),
        }
    }
}
impl TryFrom<AttributeValue> for u64 {
    type Error = error::Error;
    fn try_from(attr: AttributeValue) -> Result<Self, Self::Error> {
        match attr {
            AttributeValue::Uchar(x) => Ok((x).into()),
            AttributeValue::Schar(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Ushort(x) => Ok((x).into()),
            AttributeValue::Short(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Uint(x) => Ok((x).into()),
            AttributeValue::Int(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Ulonglong(x) => Ok(x),
            AttributeValue::Longlong(x) => (x).try_into().map_err(error::Error::Conversion),
            _ => Err("Conversion not supported".into()),
        }
    }
}
impl TryFrom<AttributeValue> for i64 {
    type Error = error::Error;
    fn try_from(attr: AttributeValue) -> Result<Self, Self::Error> {
        match attr {
            AttributeValue::Uchar(x) => Ok((x).into()),
            AttributeValue::Schar(x) => Ok((x).into()),
            AttributeValue::Ushort(x) => Ok((x).into()),
            AttributeValue::Short(x) => Ok((x).into()),
            AttributeValue::Uint(x) => Ok((x).into()),
            AttributeValue::Int(x) => Ok((x).into()),
            AttributeValue::Ulonglong(x) => (x).try_into().map_err(error::Error::Conversion),
            AttributeValue::Longlong(x) => Ok(x),
            _ => Err("Conversion not supported".into()),
        }
    }
}
impl TryFrom<AttributeValue> for f32 {
    type Error = error::Error;
    fn try_from(attr: AttributeValue) -> Result<Self, Self::Error> {
        match attr {
            AttributeValue::Uchar(x) => Ok(x as _),
            AttributeValue::Schar(x) => Ok(x as _),
            AttributeValue::Ushort(x) => Ok(x as _),
            AttributeValue::Short(x) => Ok(x as _),
            AttributeValue::Uint(x) => Ok(x as _),
            AttributeValue::Int(x) => Ok(x as _),
            AttributeValue::Ulonglong(x) => Ok(x as _),
            AttributeValue::Longlong(x) => Ok(x as _),
            AttributeValue::Float(x) => Ok(x),
            AttributeValue::Double(x) => Ok(x as _),
            _ => Err("Conversion not supported".into()),
        }
    }
}
impl TryFrom<AttributeValue> for f64 {
    type Error = error::Error;
    fn try_from(attr: AttributeValue) -> Result<Self, Self::Error> {
        match attr {
            AttributeValue::Uchar(x) => Ok(x as _),
            AttributeValue::Schar(x) => Ok(x as _),
            AttributeValue::Ushort(x) => Ok(x as _),
            AttributeValue::Short(x) => Ok(x as _),
            AttributeValue::Uint(x) => Ok(x as _),
            AttributeValue::Int(x) => Ok(x as _),
            AttributeValue::Ulonglong(x) => Ok(x as _),
            AttributeValue::Longlong(x) => Ok(x as _),
            AttributeValue::Float(x) => Ok(x as _),
            AttributeValue::Double(x) => Ok(x),
            _ => Err("Conversion not supported".into()),
        }
    }
}

impl TryFrom<AttributeValue> for String {
    type Error = error::Error;
    fn try_from(attr: AttributeValue) -> Result<Self, Self::Error> {
        match attr {
            AttributeValue::Str(s) => Ok(s),
            _ => Err("Conversion not supported".into()),
        }
    }
}

#[test]
fn roundtrip_attrvalue() {
    let x: u8 = 5;
    let attr: AttributeValue = x.into();
    assert_eq!(x, attr.try_into().unwrap());

    let x: f32 = 5.0;
    let attr: AttributeValue = x.into();
    assert_eq!(x, attr.try_into().unwrap());
}
