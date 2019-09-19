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
    pub(crate) value: Option<AnyValue>,
}

impl Attribute {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn get_value(&self) -> AnyValue {
        // This should lazy read the value when implementing reading
        self.value.clone().unwrap()
    }
}

#[derive(Debug, Clone)]
pub enum AnyValue {
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
        val: AnyValue,
    ) -> error::Result<Attribute> {
        let cname: CString = CString::new(name).unwrap();

        let _l = LOCK.lock().unwrap();
        let err;
        unsafe {
            match val {
                AnyValue::Uchar(x) => {
                    err = nc_put_att_uchar(ncid, varid, cname.as_ptr(), NC_CHAR, 1, &x);
                }
                AnyValue::Schar(x) => {
                    err = nc_put_att_schar(ncid, varid, cname.as_ptr(), NC_BYTE, 1, &x);
                }
                AnyValue::Ushort(x) => {
                    err = nc_put_att_ushort(ncid, varid, cname.as_ptr(), NC_USHORT, 1, &x);
                }
                AnyValue::Short(x) => {
                    err = nc_put_att_short(ncid, varid, cname.as_ptr(), NC_SHORT, 1, &x);
                }
                AnyValue::Uint(x) => {
                    err = nc_put_att_uint(ncid, varid, cname.as_ptr(), NC_UINT, 1, &x);
                }
                AnyValue::Int(x) => {
                    err = nc_put_att_int(ncid, varid, cname.as_ptr(), NC_INT, 1, &x);
                }
                AnyValue::Ulonglong(x) => {
                    err = nc_put_att_ulonglong(ncid, varid, cname.as_ptr(), NC_UINT64, 1, &x);
                }
                AnyValue::Longlong(x) => {
                    err = nc_put_att_longlong(ncid, varid, cname.as_ptr(), NC_INT64, 1, &x);
                }
                AnyValue::Float(x) => {
                    err = nc_put_att_float(ncid, varid, cname.as_ptr(), NC_FLOAT, 1, &x);
                }
                AnyValue::Double(x) => {
                    err = nc_put_att_double(ncid, varid, cname.as_ptr(), NC_FLOAT, 1, &x);
                }
                AnyValue::Str(ref x) => {
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
            ncid: ncid,
            varid: varid,
            value: Some(val),
        })
    }
}

// Boring implementations
impl From<u8> for AnyValue {
    fn from(x: u8) -> AnyValue {
        AnyValue::Uchar(x)
    }
}
impl From<i8> for AnyValue {
    fn from(x: i8) -> AnyValue {
        AnyValue::Schar(x)
    }
}
impl From<u16> for AnyValue {
    fn from(x: u16) -> AnyValue {
        AnyValue::Ushort(x)
    }
}
impl From<i16> for AnyValue {
    fn from(x: i16) -> AnyValue {
        AnyValue::Short(x)
    }
}
impl From<u32> for AnyValue {
    fn from(x: u32) -> AnyValue {
        AnyValue::Uint(x)
    }
}
impl From<i32> for AnyValue {
    fn from(x: i32) -> AnyValue {
        AnyValue::Int(x)
    }
}
impl From<u64> for AnyValue {
    fn from(x: u64) -> AnyValue {
        AnyValue::Ulonglong(x)
    }
}
impl From<i64> for AnyValue {
    fn from(x: i64) -> AnyValue {
        AnyValue::Longlong(x)
    }
}
impl From<f32> for AnyValue {
    fn from(x: f32) -> AnyValue {
        AnyValue::Float(x)
    }
}
impl From<f64> for AnyValue {
    fn from(x: f64) -> AnyValue {
        AnyValue::Double(x)
    }
}
impl From<&str> for AnyValue {
    fn from(x: &str) -> AnyValue {
        AnyValue::Str(x.to_string())
    }
}
impl From<String> for AnyValue {
    fn from(x: String) -> AnyValue {
        AnyValue::Str(x)
    }
}

#[test]
fn conversion() {
    let x = 1.0f32;
    let b: AnyValue = x.into();
}
