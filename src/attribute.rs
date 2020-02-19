//! Add and read attributes from netcdf groups and variables

#![allow(clippy::similar_names)]
use super::error;
use netcdf_sys::*;
use std::convert::TryInto;
use std::marker::PhantomData;

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

impl<'a> std::fmt::Debug for Attribute<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: {}", self.name())?;
        write!(f, "ncid: {}", self.ncid)?;
        write!(f, "varid: {}", self.varid)
    }
}

impl<'a> Attribute<'a> {
    /// Get the name of the attribute
    ///
    /// # Errors
    /// attribute could have a name containing an invalid utf8-sequence
    pub fn name(&self) -> &str {
        let zeropos = self
            .name
            .iter()
            .position(|&x| x == 0)
            .unwrap_or_else(|| self.name.len());
        std::str::from_utf8(&self.name[..zeropos])
            .expect("Attribute name contains invalid sequence")
    }
    /// Number of elements in this attribute
    fn num_elems(&self) -> error::Result<usize> {
        let mut nelems = 0;
        unsafe {
            error::checked(super::with_lock(|| {
                nc_inq_attlen(
                    self.ncid,
                    self.varid,
                    self.name.as_ptr() as *const _,
                    &mut nelems,
                )
            }))?;
        }
        Ok(nelems as _)
    }
    /// Type of this attribute
    fn typ(&self) -> error::Result<nc_type> {
        let mut atttype = 0;
        unsafe {
            error::checked(super::with_lock(|| {
                nc_inq_atttype(
                    self.ncid,
                    self.varid,
                    self.name.as_ptr() as *const _,
                    &mut atttype,
                )
            }))?;
        }
        Ok(atttype)
    }
    /// Get the value of the attribute
    ///
    /// # Errors
    ///
    /// Unsupported type or netcdf error
    #[allow(clippy::too_many_lines)]
    pub fn value(&self) -> error::Result<AttrValue> {
        let attlen = self.num_elems()?;
        let typ = self.typ()?;

        match typ {
            NC_UBYTE => match attlen {
                1 => {
                    let mut value = 0;
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_uchar(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                &mut value,
                            )
                        }))?;
                    }
                    Ok(AttrValue::Uchar(value))
                }
                len => {
                    let mut values = vec![0_u8; len as usize];
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_uchar(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                values.as_mut_ptr(),
                            )
                        }))?;
                    }
                    Ok(AttrValue::Uchars(values))
                }
            },
            NC_BYTE => match attlen {
                1 => {
                    let mut value = 0;
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_schar(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                &mut value,
                            )
                        }))?;
                    }
                    Ok(AttrValue::Schar(value))
                }
                len => {
                    let mut values = vec![0; len as _];
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_schar(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                values.as_mut_ptr(),
                            )
                        }))?;
                    }
                    Ok(AttrValue::Schars(values))
                }
            },
            NC_SHORT => match attlen {
                1 => {
                    let mut value = 0;
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_short(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                &mut value,
                            )
                        }))?;
                    }
                    Ok(AttrValue::Short(value))
                }
                len => {
                    let mut values = vec![0; len as _];
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_short(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                values.as_mut_ptr(),
                            )
                        }))?;
                    }
                    Ok(AttrValue::Shorts(values))
                }
            },
            NC_USHORT => match attlen {
                1 => {
                    let mut value = 0;
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_ushort(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                &mut value,
                            )
                        }))?;
                    }
                    Ok(AttrValue::Ushort(value))
                }
                len => {
                    let mut values = vec![0; len as _];
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_ushort(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                values.as_mut_ptr(),
                            )
                        }))?;
                    }
                    Ok(AttrValue::Ushorts(values))
                }
            },
            NC_INT => match attlen {
                1 => {
                    let mut value = 0;
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_int(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                &mut value,
                            )
                        }))?;
                    }
                    Ok(AttrValue::Int(value))
                }
                len => {
                    let mut values = vec![0; len as _];
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_int(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                values.as_mut_ptr(),
                            )
                        }))?;
                    }
                    Ok(AttrValue::Ints(values))
                }
            },
            NC_UINT => match attlen {
                1 => {
                    let mut value = 0;
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_uint(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                &mut value,
                            )
                        }))?;
                    }
                    Ok(AttrValue::Uint(value))
                }
                len => {
                    let mut values = vec![0; len as _];
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_uint(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                values.as_mut_ptr(),
                            )
                        }))?;
                    }
                    Ok(AttrValue::Uints(values))
                }
            },
            NC_INT64 => match attlen {
                1 => {
                    let mut value = 0;
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_longlong(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                &mut value,
                            )
                        }))?;
                    }
                    Ok(AttrValue::Longlong(value))
                }
                len => {
                    let mut values = vec![0; len as _];
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_longlong(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                values.as_mut_ptr(),
                            )
                        }))?;
                    }

                    Ok(AttrValue::Longlongs(values))
                }
            },
            NC_UINT64 => match attlen {
                1 => {
                    let mut value = 0;
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_ulonglong(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                &mut value,
                            )
                        }))?;
                    }
                    Ok(AttrValue::Ulonglong(value))
                }
                len => {
                    let mut values = vec![0; len as _];
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_ulonglong(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                values.as_mut_ptr(),
                            )
                        }))?;
                    }

                    Ok(AttrValue::Ulonglongs(values))
                }
            },
            NC_FLOAT => match attlen {
                1 => {
                    let mut value = 0.0;
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_float(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                &mut value,
                            )
                        }))?;
                    }
                    Ok(AttrValue::Float(value))
                }
                len => {
                    let mut values = vec![0.0; len as _];
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_float(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                values.as_mut_ptr(),
                            )
                        }))?;
                    }
                    Ok(AttrValue::Floats(values))
                }
            },
            NC_DOUBLE => match attlen {
                1 => {
                    let mut value = 0.0;
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_double(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                &mut value,
                            )
                        }))?;
                    }
                    Ok(AttrValue::Double(value))
                }
                len => {
                    let mut values = vec![0.0; len as _];
                    unsafe {
                        error::checked(super::with_lock(|| {
                            nc_get_att_double(
                                self.ncid,
                                self.varid,
                                self.name.as_ptr() as *const _,
                                values.as_mut_ptr(),
                            )
                        }))?;
                    }
                    Ok(AttrValue::Doubles(values))
                }
            },
            NC_CHAR => {
                let lentext = attlen;
                let mut buf: Vec<u8> = vec![0; lentext as _];
                unsafe {
                    error::checked(super::with_lock(|| {
                        nc_get_att_text(
                            self.ncid,
                            self.varid,
                            self.name.as_ptr() as *const _,
                            buf.as_mut_ptr() as *mut _,
                        )
                    }))?;
                }
                let pos = buf
                    .iter()
                    .position(|&x| x == 0)
                    .unwrap_or_else(|| buf.len());
                Ok(AttrValue::Str(String::from(String::from_utf8_lossy(
                    &buf[..pos],
                ))))
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

impl<'a> AttributeIterator<'a> {
    pub(crate) fn new(ncid: nc_type, varid: Option<nc_type>) -> error::Result<Self> {
        let mut natts = 0;
        unsafe {
            error::checked(super::with_lock(|| {
                nc_inq_varnatts(ncid, varid.unwrap_or(NC_GLOBAL), &mut natts)
            }))?;
        }
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
        unsafe {
            if let Err(e) = error::checked(super::with_lock(|| {
                nc_inq_attname(
                    self.ncid,
                    self.varid.unwrap_or(NC_GLOBAL),
                    self.current_natt.try_into().unwrap(),
                    name.as_mut_ptr() as *mut _,
                )
            })) {
                return Some(Err(e));
            }
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
pub enum AttrValue {
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
}

impl<'a> Attribute<'a> {
    #[allow(clippy::needless_pass_by_value)] // All values will be small
    #[allow(clippy::too_many_lines)]
    pub(crate) fn put(
        ncid: nc_type,
        varid: nc_type,
        name: &str,
        val: AttrValue,
    ) -> error::Result<Self> {
        let cname = super::utils::short_name_to_bytes(name)?;

        error::checked(unsafe {
            match val {
                AttrValue::Uchar(x) => super::with_lock(|| {
                    nc_put_att_uchar(ncid, varid, cname.as_ptr() as *const _, NC_UBYTE, 1, &x)
                }),
                AttrValue::Uchars(x) => super::with_lock(|| {
                    nc_put_att_uchar(
                        ncid,
                        varid,
                        cname.as_ptr() as *const _,
                        NC_UBYTE,
                        x.len(),
                        x.as_ptr(),
                    )
                }),
                AttrValue::Schar(x) => super::with_lock(|| {
                    nc_put_att_schar(ncid, varid, cname.as_ptr() as *const _, NC_BYTE, 1, &x)
                }),
                AttrValue::Schars(x) => super::with_lock(|| {
                    nc_put_att_schar(
                        ncid,
                        varid,
                        cname.as_ptr() as *const _,
                        NC_BYTE,
                        x.len(),
                        x.as_ptr(),
                    )
                }),
                AttrValue::Ushort(x) => super::with_lock(|| {
                    nc_put_att_ushort(ncid, varid, cname.as_ptr() as *const _, NC_USHORT, 1, &x)
                }),
                AttrValue::Ushorts(x) => super::with_lock(|| {
                    nc_put_att_ushort(
                        ncid,
                        varid,
                        cname.as_ptr() as *const _,
                        NC_USHORT,
                        x.len(),
                        x.as_ptr(),
                    )
                }),
                AttrValue::Short(x) => super::with_lock(|| {
                    nc_put_att_short(ncid, varid, cname.as_ptr() as *const _, NC_SHORT, 1, &x)
                }),
                AttrValue::Shorts(x) => super::with_lock(|| {
                    nc_put_att_short(
                        ncid,
                        varid,
                        cname.as_ptr() as *const _,
                        NC_SHORT,
                        x.len(),
                        x.as_ptr(),
                    )
                }),
                AttrValue::Uint(x) => super::with_lock(|| {
                    nc_put_att_uint(ncid, varid, cname.as_ptr() as *const _, NC_UINT, 1, &x)
                }),
                AttrValue::Uints(x) => super::with_lock(|| {
                    nc_put_att_uint(
                        ncid,
                        varid,
                        cname.as_ptr() as *const _,
                        NC_UINT,
                        x.len(),
                        x.as_ptr(),
                    )
                }),
                AttrValue::Int(x) => super::with_lock(|| {
                    nc_put_att_int(ncid, varid, cname.as_ptr() as *const _, NC_INT, 1, &x)
                }),
                AttrValue::Ints(x) => super::with_lock(|| {
                    nc_put_att_int(
                        ncid,
                        varid,
                        cname.as_ptr() as *const _,
                        NC_INT,
                        x.len(),
                        x.as_ptr(),
                    )
                }),
                AttrValue::Ulonglong(x) => super::with_lock(|| {
                    nc_put_att_ulonglong(ncid, varid, cname.as_ptr() as *const _, NC_UINT64, 1, &x)
                }),
                AttrValue::Ulonglongs(x) => super::with_lock(|| {
                    nc_put_att_ulonglong(
                        ncid,
                        varid,
                        cname.as_ptr() as *const _,
                        NC_UINT64,
                        x.len(),
                        x.as_ptr(),
                    )
                }),
                AttrValue::Longlong(x) => super::with_lock(|| {
                    nc_put_att_longlong(ncid, varid, cname.as_ptr() as *const _, NC_INT64, 1, &x)
                }),
                AttrValue::Longlongs(x) => super::with_lock(|| {
                    nc_put_att_longlong(
                        ncid,
                        varid,
                        cname.as_ptr() as *const _,
                        NC_INT64,
                        x.len(),
                        x.as_ptr(),
                    )
                }),
                AttrValue::Float(x) => super::with_lock(|| {
                    nc_put_att_float(ncid, varid, cname.as_ptr() as *const _, NC_FLOAT, 1, &x)
                }),
                AttrValue::Floats(x) => super::with_lock(|| {
                    nc_put_att_float(
                        ncid,
                        varid,
                        cname.as_ptr() as *const _,
                        NC_FLOAT,
                        x.len(),
                        x.as_ptr(),
                    )
                }),
                AttrValue::Double(x) => super::with_lock(|| {
                    nc_put_att_double(ncid, varid, cname.as_ptr() as *const _, NC_DOUBLE, 1, &x)
                }),
                AttrValue::Doubles(x) => super::with_lock(|| {
                    nc_put_att_double(
                        ncid,
                        varid,
                        cname.as_ptr() as *const _,
                        NC_DOUBLE,
                        x.len(),
                        x.as_ptr(),
                    )
                }),
                AttrValue::Str(ref x) => super::with_lock(|| {
                    nc_put_att_text(
                        ncid,
                        varid,
                        cname.as_ptr() as *const _,
                        x.len(),
                        x.as_ptr() as *const _,
                    )
                }),
            }
        })?;

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
        let e = unsafe {
            // Checking whether the variable exists by probing for its id
            super::with_lock(|| {
                nc_inq_attid(
                    ncid,
                    varid.unwrap_or(NC_GLOBAL),
                    attname.as_ptr() as *const _,
                    std::ptr::null_mut(),
                )
            })
        };
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
impl From<u8> for AttrValue {
    fn from(x: u8) -> Self {
        Self::Uchar(x)
    }
}
impl From<Vec<u8>> for AttrValue {
    fn from(x: Vec<u8>) -> Self {
        Self::Uchars(x)
    }
}
impl From<i8> for AttrValue {
    fn from(x: i8) -> Self {
        Self::Schar(x)
    }
}
impl From<Vec<i8>> for AttrValue {
    fn from(x: Vec<i8>) -> Self {
        Self::Schars(x)
    }
}
impl From<u16> for AttrValue {
    fn from(x: u16) -> Self {
        Self::Ushort(x)
    }
}
impl From<Vec<u16>> for AttrValue {
    fn from(x: Vec<u16>) -> Self {
        Self::Ushorts(x)
    }
}
impl From<i16> for AttrValue {
    fn from(x: i16) -> Self {
        Self::Short(x)
    }
}
impl From<Vec<i16>> for AttrValue {
    fn from(x: Vec<i16>) -> Self {
        Self::Shorts(x)
    }
}
impl From<u32> for AttrValue {
    fn from(x: u32) -> Self {
        Self::Uint(x)
    }
}
impl From<Vec<u32>> for AttrValue {
    fn from(x: Vec<u32>) -> Self {
        Self::Uints(x)
    }
}
impl From<i32> for AttrValue {
    fn from(x: i32) -> Self {
        Self::Int(x)
    }
}
impl From<Vec<i32>> for AttrValue {
    fn from(x: Vec<i32>) -> Self {
        Self::Ints(x)
    }
}
impl From<u64> for AttrValue {
    fn from(x: u64) -> Self {
        Self::Ulonglong(x)
    }
}
impl From<Vec<u64>> for AttrValue {
    fn from(x: Vec<u64>) -> Self {
        Self::Ulonglongs(x)
    }
}
impl From<i64> for AttrValue {
    fn from(x: i64) -> Self {
        Self::Longlong(x)
    }
}
impl From<Vec<i64>> for AttrValue {
    fn from(x: Vec<i64>) -> Self {
        Self::Longlongs(x)
    }
}
impl From<f32> for AttrValue {
    fn from(x: f32) -> Self {
        Self::Float(x)
    }
}
impl From<Vec<f32>> for AttrValue {
    fn from(x: Vec<f32>) -> Self {
        Self::Floats(x)
    }
}
impl From<f64> for AttrValue {
    fn from(x: f64) -> Self {
        Self::Double(x)
    }
}
impl From<Vec<f64>> for AttrValue {
    fn from(x: Vec<f64>) -> Self {
        Self::Doubles(x)
    }
}
impl From<&str> for AttrValue {
    fn from(x: &str) -> Self {
        Self::Str(x.to_string())
    }
}
impl From<String> for AttrValue {
    fn from(x: String) -> Self {
        Self::Str(x)
    }
}

#[test]
fn conversion() {
    let x = 1.0f32;
    let _b: AttrValue = x.into();
}
