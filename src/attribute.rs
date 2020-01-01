//! Add and read attributes from netcdf groups and variables

#![allow(clippy::similar_names)]
use super::error;
use super::LOCK;
use netcdf_sys::*;
use std::marker::PhantomData;

/// Extra properties of a variable or a group can be represented
/// with attributes. Primarily added with `add_attribute` on
/// the variable and group
#[derive(Clone)]
pub struct Attribute<'a> {
    pub(crate) name: [u8; NC_MAX_NAME as usize + 1],
    /// Group or file this attribute is in
    pub(crate) ncid: nc_type,
    /// Variable/global this id is connected to
    pub(crate) varid: nc_type,
    /// Attribute type
    pub(crate) atttype: nc_type,
    /// Vector length
    pub(crate) attlen: nc_type,
    /// Holds the variable/group to prevent the
    /// attribute being deleted or modified
    pub(crate) _marker: PhantomData<&'a nc_type>,
}

impl<'a> std::fmt::Debug for Attribute<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: ")?;
        if let Ok(name) = self.name() {
            write!(f, "{}", name)?;
        } else {
            write!(f, "<<not utf8 name>>")?;
        }
        write!(f, "ncid: {}", self.ncid)?;
        write!(f, "varid: {}", self.varid)?;
        write!(f, "atttype: {}", self.atttype)?;
        write!(f, "attlen: {}", self.attlen)
    }
}

impl<'a> Attribute<'a> {
    /// Get the name of the attribute
    ///
    /// # Errors
    /// attribute could have a name containing an invalid utf8-sequence
    pub fn name(&self) -> Result<&str, std::str::Utf8Error> {
        let zeropos = self
            .name
            .iter()
            .position(|&x| x == 0)
            .unwrap_or(self.name.len());
        std::str::from_utf8(&self.name[..zeropos])
    }
    /// Get the value of the attribute
    #[allow(clippy::too_many_lines)]
    pub fn value(&self) -> error::Result<AttrValue> {
        let mut typ = 0;
        let _l = LOCK.lock().unwrap();
        unsafe {
            error::checked(nc_inq_atttype(
                self.ncid,
                self.varid,
                self.name.as_ptr() as *const _,
                &mut typ,
            ))?;
        }

        match typ {
            NC_UBYTE => {
                let mut value = 0;
                unsafe {
                    error::checked(nc_get_att_uchar(
                        self.ncid,
                        self.varid,
                        self.name.as_ptr() as *const _,
                        &mut value,
                    ))?;
                }
                Ok(AttrValue::Uchar(value))
            }
            NC_BYTE => {
                let mut value = 0;
                unsafe {
                    error::checked(nc_get_att_schar(
                        self.ncid,
                        self.varid,
                        self.name.as_ptr() as *const _,
                        &mut value,
                    ))?;
                }
                Ok(AttrValue::Schar(value))
            }
            NC_SHORT => {
                let mut value = 0;
                unsafe {
                    error::checked(nc_get_att_short(
                        self.ncid,
                        self.varid,
                        self.name.as_ptr() as *const _,
                        &mut value,
                    ))?;
                }
                Ok(AttrValue::Short(value))
            }
            NC_USHORT => {
                let mut value = 0;
                unsafe {
                    error::checked(nc_get_att_ushort(
                        self.ncid,
                        self.varid,
                        self.name.as_ptr() as *const _,
                        &mut value,
                    ))?;
                }
                Ok(AttrValue::Ushort(value))
            }
            NC_INT => {
                let mut value = 0;
                unsafe {
                    error::checked(nc_get_att_int(
                        self.ncid,
                        self.varid,
                        self.name.as_ptr() as *const _,
                        &mut value,
                    ))?;
                }
                Ok(AttrValue::Int(value))
            }
            NC_UINT => {
                let mut value = 0;
                unsafe {
                    error::checked(nc_get_att_uint(
                        self.ncid,
                        self.varid,
                        self.name.as_ptr() as *const _,
                        &mut value,
                    ))?;
                }
                Ok(AttrValue::Uint(value))
            }
            NC_INT64 => {
                let mut value = 0;
                unsafe {
                    error::checked(nc_get_att_longlong(
                        self.ncid,
                        self.varid,
                        self.name.as_ptr() as *const _,
                        &mut value,
                    ))?;
                }
                Ok(AttrValue::Longlong(value))
            }
            NC_UINT64 => {
                let mut value = 0;
                unsafe {
                    error::checked(nc_get_att_ulonglong(
                        self.ncid,
                        self.varid,
                        self.name.as_ptr() as *const _,
                        &mut value,
                    ))?;
                }
                Ok(AttrValue::Ulonglong(value))
            }
            NC_FLOAT => {
                let mut value = 0.0;
                unsafe {
                    error::checked(nc_get_att_float(
                        self.ncid,
                        self.varid,
                        self.name.as_ptr() as *const _,
                        &mut value,
                    ))?;
                }
                Ok(AttrValue::Float(value))
            }
            NC_DOUBLE => {
                let mut value = 0.0;
                unsafe {
                    error::checked(nc_get_att_double(
                        self.ncid,
                        self.varid,
                        self.name.as_ptr() as *const _,
                        &mut value,
                    ))?;
                }
                Ok(AttrValue::Double(value))
            }
            NC_CHAR => {
                let mut lentext = 0;
                unsafe {
                    error::checked(nc_inq_attlen(
                        self.ncid,
                        self.varid,
                        self.name.as_ptr() as *const _,
                        &mut lentext,
                    ))?;
                }
                let mut buf: Vec<u8> = vec![0; lentext];
                unsafe {
                    error::checked(nc_get_att_text(
                        self.ncid,
                        self.varid,
                        self.name.as_ptr() as *const _,
                        buf.as_mut_ptr() as *mut _,
                    ))?;
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

pub(crate) struct AttributeIterator<'a> {
    ncid: nc_type,
    varid: Option<nc_type>,
    natts: usize,
    current_natt: usize,
    _marker: PhantomData<&'a nc_type>,
}

impl<'a> AttributeIterator<'a> {
    pub(crate) fn new(ncid: nc_type, varid: Option<nc_type>) -> error::Result<Self> {
        let _l = LOCK.lock().unwrap();
        let mut natts = 0;
        unsafe {
            error::checked(nc_inq_varnatts(
                ncid,
                varid.unwrap_or(NC_GLOBAL),
                &mut natts,
            ))?;
        }
        Ok(Self {
            ncid,
            varid,
            natts: natts as _,
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

        let _l = LOCK.lock().unwrap();
        let mut name = [0_u8; NC_MAX_NAME as usize + 1];
        unsafe {
            if let Err(e) = error::checked(nc_inq_attname(
                self.ncid,
                self.varid.unwrap_or(NC_GLOBAL),
                self.current_natt as _,
                name.as_mut_ptr() as *mut _,
            )) {
                return Some(Err(e));
            }
        }
        let mut atttype = 0;
        unsafe {
            if let Err(e) = error::checked(nc_inq_atttype(
                self.ncid,
                self.varid.unwrap_or(NC_GLOBAL),
                name.as_ptr() as *const _,
                &mut atttype,
            )) {
                return Some(Err(e));
            }
        }
        let mut attlen = 0;
        unsafe {
            if let Err(e) = error::checked(nc_inq_attlen(
                self.ncid,
                self.varid.unwrap_or(NC_GLOBAL),
                name.as_ptr() as *const _,
                &mut attlen,
            )) {
                return Some(Err(e));
            }
        }

        let att = Attribute {
            name,
            ncid: self.ncid,
            varid: self.varid.unwrap_or(NC_GLOBAL),
            attlen: attlen as _,
            atttype,
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
    Schar(i8),
    Ushort(u16),
    Short(i16),
    Uint(u32),
    Int(i32),
    Ulonglong(u64),
    Longlong(i64),
    Float(f32),
    Double(f64),
    Str(String),
}

impl<'a> Attribute<'a> {
    #[allow(clippy::needless_pass_by_value)] // All values will be small
    pub(crate) fn put(
        ncid: nc_type,
        varid: nc_type,
        name: &str,
        val: AttrValue,
    ) -> error::Result<Self> {
        let cname = {
            if name.len() > NC_MAX_NAME as usize {
                return Err(error::Error::Netcdf(NC_EMAXNAME));
            }
            let mut attname = [0_u8; NC_MAX_NAME as usize + 1];
            attname[..name.len()].copy_from_slice(name.as_bytes());
            attname
        };

        let _l = LOCK.lock().unwrap();
        let atttype;
        error::checked(unsafe {
            match val {
                AttrValue::Uchar(x) => {
                    atttype = NC_UBYTE;
                    nc_put_att_uchar(ncid, varid, cname.as_ptr() as *const _, atttype, 1, &x)
                }
                AttrValue::Schar(x) => {
                    atttype = NC_BYTE;
                    nc_put_att_schar(ncid, varid, cname.as_ptr() as *const _, atttype, 1, &x)
                }
                AttrValue::Ushort(x) => {
                    atttype = NC_USHORT;
                    nc_put_att_ushort(ncid, varid, cname.as_ptr() as *const _, atttype, 1, &x)
                }
                AttrValue::Short(x) => {
                    atttype = NC_SHORT;
                    nc_put_att_short(ncid, varid, cname.as_ptr() as *const _, atttype, 1, &x)
                }
                AttrValue::Uint(x) => {
                    atttype = NC_UINT;
                    nc_put_att_uint(ncid, varid, cname.as_ptr() as *const _, atttype, 1, &x)
                }
                AttrValue::Int(x) => {
                    atttype = NC_INT;
                    nc_put_att_int(ncid, varid, cname.as_ptr() as *const _, atttype, 1, &x)
                }
                AttrValue::Ulonglong(x) => {
                    atttype = NC_UINT64;
                    nc_put_att_ulonglong(ncid, varid, cname.as_ptr() as *const _, atttype, 1, &x)
                }
                AttrValue::Longlong(x) => {
                    atttype = NC_INT64;
                    nc_put_att_longlong(ncid, varid, cname.as_ptr() as *const _, atttype, 1, &x)
                }
                AttrValue::Float(x) => {
                    atttype = NC_FLOAT;
                    nc_put_att_float(ncid, varid, cname.as_ptr() as *const _, atttype, 1, &x)
                }
                AttrValue::Double(x) => {
                    atttype = NC_DOUBLE;
                    nc_put_att_double(ncid, varid, cname.as_ptr() as *const _, atttype, 1, &x)
                }
                AttrValue::Str(ref x) => {
                    atttype = NC_STRING;
                    nc_put_att_text(
                        ncid,
                        varid,
                        cname.as_ptr() as *const _,
                        x.len(),
                        x.as_ptr() as *const _,
                    )
                }
            }
        })?;

        Ok(Self {
            name: cname,
            ncid,
            varid,
            atttype,
            attlen: 1,
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
        let _l = LOCK.lock().unwrap();
        let e;
        let mut xtype = 0;
        unsafe {
            e = nc_inq_atttype(
                ncid,
                varid.unwrap_or(NC_GLOBAL),
                attname.as_ptr() as *const _,
                &mut xtype,
            );
        }
        if e == NC_ENOTATT {
            return Ok(None);
        }
        error::checked(e)?;
        let mut xlen = 0;
        unsafe {
            error::checked(nc_inq_attlen(
                ncid,
                varid.unwrap_or(NC_GLOBAL),
                attname.as_ptr() as *const _,
                &mut xlen,
            ))?;
        }

        Ok(Some(Attribute {
            name: attname,
            ncid: ncid,
            varid: varid.unwrap_or(NC_GLOBAL),
            atttype: xtype,
            attlen: xlen as _,
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
impl From<i8> for AttrValue {
    fn from(x: i8) -> Self {
        Self::Schar(x)
    }
}
impl From<u16> for AttrValue {
    fn from(x: u16) -> Self {
        Self::Ushort(x)
    }
}
impl From<i16> for AttrValue {
    fn from(x: i16) -> Self {
        Self::Short(x)
    }
}
impl From<u32> for AttrValue {
    fn from(x: u32) -> Self {
        Self::Uint(x)
    }
}
impl From<i32> for AttrValue {
    fn from(x: i32) -> Self {
        Self::Int(x)
    }
}
impl From<u64> for AttrValue {
    fn from(x: u64) -> Self {
        Self::Ulonglong(x)
    }
}
impl From<i64> for AttrValue {
    fn from(x: i64) -> Self {
        Self::Longlong(x)
    }
}
impl From<f32> for AttrValue {
    fn from(x: f32) -> Self {
        Self::Float(x)
    }
}
impl From<f64> for AttrValue {
    fn from(x: f64) -> Self {
        Self::Double(x)
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
