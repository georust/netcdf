use super::error;
use super::LOCK;
use netcdf_sys::*;
use std::ffi::CString;

#[derive(Debug)]
pub struct Attribute {
    pub(crate) name: String,
    /// Group or file this attribute is in
    pub(crate) ncid: nc_type,
    /// Variable/global this id is connected to
    pub(crate) varid: nc_type,
}

impl Attribute {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn value(&self) -> error::Result<AttrValue> {
        let mut typ = 0;
        let err;
        let cname = std::ffi::CString::new(self.name.clone()).unwrap();
        unsafe {
            err = nc_inq_atttype(self.ncid, self.varid, cname.as_ptr(), &mut typ);
        }
        if err != NC_NOERR {
            return Err(err.into());
        }
        match typ {
            NC_UBYTE => {
                let mut value = 0;
                let err;
                unsafe {
                    err = nc_get_att_uchar(self.ncid, self.varid, cname.as_ptr(), &mut value);
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }
                Ok(AttrValue::Uchar(value))
            }
            NC_BYTE => {
                let mut value = 0;
                let err;
                unsafe {
                    err = nc_get_att_schar(self.ncid, self.varid, cname.as_ptr(), &mut value);
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }
                Ok(AttrValue::Schar(value))
            }
            NC_SHORT => {
                let mut value = 0;
                let err;
                unsafe {
                    err = nc_get_att_short(self.ncid, self.varid, cname.as_ptr(), &mut value);
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }
                Ok(AttrValue::Short(value))
            }
            NC_USHORT => {
                let mut value = 0;
                let err;
                unsafe {
                    err = nc_get_att_ushort(self.ncid, self.varid, cname.as_ptr(), &mut value);
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }
                Ok(AttrValue::Ushort(value))
            }
            NC_INT => {
                let mut value = 0;
                let err;
                unsafe {
                    err = nc_get_att_int(self.ncid, self.varid, cname.as_ptr(), &mut value);
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }
                Ok(AttrValue::Int(value))
            }
            NC_UINT => {
                let mut value = 0;
                let err;
                unsafe {
                    err = nc_get_att_uint(self.ncid, self.varid, cname.as_ptr(), &mut value);
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }
                Ok(AttrValue::Uint(value))
            }
            NC_INT64 => {
                let mut value = 0;
                let err;
                unsafe {
                    err = nc_get_att_longlong(self.ncid, self.varid, cname.as_ptr(), &mut value);
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }
                Ok(AttrValue::Longlong(value))
            }
            NC_UINT64 => {
                let mut value = 0;
                let err;
                unsafe {
                    err = nc_get_att_ulonglong(self.ncid, self.varid, cname.as_ptr(), &mut value);
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }
                Ok(AttrValue::Ulonglong(value))
            }
            NC_FLOAT => {
                let mut value = 0.0;
                let err;
                unsafe {
                    err = nc_get_att_float(self.ncid, self.varid, cname.as_ptr(), &mut value);
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }
                Ok(AttrValue::Float(value))
            }
            NC_DOUBLE => {
                let mut value = 0.0;
                let err;
                unsafe {
                    err = nc_get_att_double(self.ncid, self.varid, cname.as_ptr(), &mut value);
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }
                Ok(AttrValue::Double(value))
            }
            NC_CHAR => {
                let mut lentext = 0;
                let err;
                unsafe {
                    err = nc_inq_attlen(self.ncid, self.varid, cname.as_ptr(), &mut lentext);
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }
                let mut buf = vec![0; lentext];
                let err;
                unsafe {
                    err = nc_get_att_text(self.ncid, self.varid, cname.as_ptr(), buf.as_mut_ptr());
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }
                let value = buf
                    .into_iter()
                    .take_while(|x| *x != 0)
                    .map(|x| x as _)
                    .collect::<Vec<_>>();
                Ok(AttrValue::Str(String::from_utf8(value).unwrap()))
            }
            x => Err(format!("Unknown nc_type encountered: {}", x).into()),
        }
    }
}

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

impl Attribute {
    pub(crate) fn put(
        ncid: nc_type,
        varid: nc_type,
        name: &str,
        val: AttrValue,
    ) -> error::Result<Attribute> {
        let cname: CString = CString::new(name).unwrap();

        let _l = LOCK.lock().unwrap();
        let err;
        unsafe {
            match val {
                AttrValue::Uchar(x) => {
                    err = nc_put_att_uchar(ncid, varid, cname.as_ptr(), NC_UBYTE, 1, &x);
                }
                AttrValue::Schar(x) => {
                    err = nc_put_att_schar(ncid, varid, cname.as_ptr(), NC_BYTE, 1, &x);
                }
                AttrValue::Ushort(x) => {
                    err = nc_put_att_ushort(ncid, varid, cname.as_ptr(), NC_USHORT, 1, &x);
                }
                AttrValue::Short(x) => {
                    err = nc_put_att_short(ncid, varid, cname.as_ptr(), NC_SHORT, 1, &x);
                }
                AttrValue::Uint(x) => {
                    err = nc_put_att_uint(ncid, varid, cname.as_ptr(), NC_UINT, 1, &x);
                }
                AttrValue::Int(x) => {
                    err = nc_put_att_int(ncid, varid, cname.as_ptr(), NC_INT, 1, &x);
                }
                AttrValue::Ulonglong(x) => {
                    err = nc_put_att_ulonglong(ncid, varid, cname.as_ptr(), NC_UINT64, 1, &x);
                }
                AttrValue::Longlong(x) => {
                    err = nc_put_att_longlong(ncid, varid, cname.as_ptr(), NC_INT64, 1, &x);
                }
                AttrValue::Float(x) => {
                    err = nc_put_att_float(ncid, varid, cname.as_ptr(), NC_FLOAT, 1, &x);
                }
                AttrValue::Double(x) => {
                    err = nc_put_att_double(ncid, varid, cname.as_ptr(), NC_DOUBLE, 1, &x);
                }
                AttrValue::Str(ref x) => {
                    err = nc_put_att_text(
                        ncid,
                        varid,
                        cname.as_ptr(),
                        x.len(),
                        x.as_ptr() as *const _,
                    );
                }
            }
        }
        if err != NC_NOERR {
            return Err(err.into());
        }

        Ok(Attribute {
            name: name.to_string(),
            ncid,
            varid,
        })
    }
}

// Boring implementations
impl From<u8> for AttrValue {
    fn from(x: u8) -> AttrValue {
        AttrValue::Uchar(x)
    }
}
impl From<i8> for AttrValue {
    fn from(x: i8) -> AttrValue {
        AttrValue::Schar(x)
    }
}
impl From<u16> for AttrValue {
    fn from(x: u16) -> AttrValue {
        AttrValue::Ushort(x)
    }
}
impl From<i16> for AttrValue {
    fn from(x: i16) -> AttrValue {
        AttrValue::Short(x)
    }
}
impl From<u32> for AttrValue {
    fn from(x: u32) -> AttrValue {
        AttrValue::Uint(x)
    }
}
impl From<i32> for AttrValue {
    fn from(x: i32) -> AttrValue {
        AttrValue::Int(x)
    }
}
impl From<u64> for AttrValue {
    fn from(x: u64) -> AttrValue {
        AttrValue::Ulonglong(x)
    }
}
impl From<i64> for AttrValue {
    fn from(x: i64) -> AttrValue {
        AttrValue::Longlong(x)
    }
}
impl From<f32> for AttrValue {
    fn from(x: f32) -> AttrValue {
        AttrValue::Float(x)
    }
}
impl From<f64> for AttrValue {
    fn from(x: f64) -> AttrValue {
        AttrValue::Double(x)
    }
}
impl From<&str> for AttrValue {
    fn from(x: &str) -> AttrValue {
        AttrValue::Str(x.to_string())
    }
}
impl From<String> for AttrValue {
    fn from(x: String) -> AttrValue {
        AttrValue::Str(x)
    }
}

#[test]
fn conversion() {
    let x = 1.0f32;
    let _b: AttrValue = x.into();
}
