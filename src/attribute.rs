use super::error;
use super::LOCK;
use netcdf_sys::*;
use std::ffi::{CStr, CString};
use std::fmt;

#[derive(Debug)]
pub struct Attribute {
    pub(crate) name: String,
    pub(crate) attrtype: nc_type,
    /// Group or file this attribute is in
    pub(crate) ncid: nc_type,
    /// Variable/global this id is connected to
    pub(crate) varid: nc_type,
}

macro_rules! get_attr_as_type {
    ( $me:ident, $nc_type:ident, $rs_type:ty, $nc_fn:ident , $cast:ident ) => {{
        if (!$cast) && ($me.attrtype != $nc_type) {
            return Err("Types are not equivalent and cast==false".into());
        }
        let mut err;
        let mut attlen: usize = 0;
        let name_copy: CString = CString::new($me.name.clone()).unwrap();
        unsafe {
            let _g = LOCK.lock().unwrap();
            err = nc_inq_attlen($me.ncid, $me.varid, name_copy.as_ptr(), &mut attlen);
        }
        if err != NC_NOERR {
            return Err(err.into());
        }
        if attlen != 1 {
            return Err("Multi-value attributes not yet implemented".into());
        }
        let mut buf: $rs_type = 0 as $rs_type;
        unsafe {
            let _g = LOCK.lock().unwrap();
            err = $nc_fn($me.ncid, $me.varid, name_copy.as_ptr(), &mut buf);
        }
        if err != NC_NOERR {
            return Err(err.into());
        }
        Ok(buf)
    }};
}

impl Attribute {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn attrtype(&self) -> nc_type {
        self.attrtype
    }
}

impl Attribute {
    pub fn get_char(&self, cast: bool) -> error::Result<String> {
        if (!cast) && (self.attrtype != NC_CHAR) {
            return Err("Types are not equivalent and cast==false".into());
        }
        let attr_char_str;
        let name_copy: CString = CString::new(self.name.clone()).unwrap();
        let mut attlen: usize = 0;
        unsafe {
            let mut err;
            {
                let _g = LOCK.lock().unwrap();
                err = nc_inq_attlen(self.ncid, self.varid, name_copy.as_ptr(), &mut attlen);
            }
            if err != NC_NOERR {
                return Err(err.into());
            }
            // careful; netcdf does not write null terminators here
            let mut attr_char_buf_vec = vec![0i8; (attlen + 1) as usize];
            let attr_char_buf_ptr: *mut i8 = attr_char_buf_vec.as_mut_ptr();
            {
                let _g = LOCK.lock().unwrap();
                err = nc_get_att_text(
                    self.ncid,
                    self.varid,
                    name_copy.as_ptr(),
                    attr_char_buf_ptr,
                );
            }
            if err != NC_NOERR {
                return Err(err.into());
            }
            let attr_c_str = CStr::from_ptr(attr_char_buf_ptr);
            attr_char_str = attr_c_str.to_string_lossy().to_string();
        }
        Ok(attr_char_str)
    }

    pub fn get_byte(&self, cast: bool) -> error::Result<i8> {
        get_attr_as_type!(self, NC_BYTE, i8, nc_get_att_schar, cast)
    }

    pub fn get_short(&self, cast: bool) -> error::Result<i16> {
        get_attr_as_type!(self, NC_SHORT, i16, nc_get_att_short, cast)
    }

    pub fn get_ushort(&self, cast: bool) -> error::Result<u16> {
        get_attr_as_type!(self, NC_USHORT, u16, nc_get_att_ushort, cast)
    }

    pub fn get_int(&self, cast: bool) -> error::Result<i32> {
        get_attr_as_type!(self, NC_INT, i32, nc_get_att_int, cast)
    }

    pub fn get_uint(&self, cast: bool) -> error::Result<u32> {
        get_attr_as_type!(self, NC_UINT, u32, nc_get_att_uint, cast)
    }

    pub fn get_int64(&self, cast: bool) -> error::Result<i64> {
        get_attr_as_type!(self, NC_INT64, i64, nc_get_att_longlong, cast)
    }

    pub fn get_uint64(&self, cast: bool) -> error::Result<u64> {
        get_attr_as_type!(self, NC_UINT64, u64, nc_get_att_ulonglong, cast)
    }

    pub fn get_float(&self, cast: bool) -> error::Result<f32> {
        get_attr_as_type!(self, NC_FLOAT, f32, nc_get_att_float, cast)
    }

    pub fn get_double(&self, cast: bool) -> error::Result<f64> {
        get_attr_as_type!(self, NC_DOUBLE, f64, nc_get_att_double, cast)
    }
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.get_char(true) {
            Ok(chars) => write!(f, "{}", chars),
            Err(e) => write!(f, "ERROR: {}", e),
        }
    }
}

// Write support for all attribute types
pub trait PutAttr {
    fn get_nc_type(&self) -> nc_type;
    fn put(&self, ncid: nc_type, varid: nc_type, name: &str) -> error::Result<()>;
}

// This macro implements the trait PutAttr for $type
// It just avoid code repetition for all numeric types
// (the only difference between each type beeing the
// netCDF funtion to call and the numeric identifier
// of the type used by the libnetCDF library)
macro_rules! impl_putattr {
    ($type: ty, $nc_type: ident, $nc_put_att: ident) => {
        impl PutAttr for $type {
            fn get_nc_type(&self) -> nc_type {
                $nc_type
            }
            fn put(&self, ncid: nc_type, varid: nc_type, name: &str) -> error::Result<()> {
                let name_c: CString = CString::new(name.clone()).unwrap();
                let err;
                unsafe {
                    let _g = LOCK.lock().unwrap();
                    err = $nc_put_att(ncid, varid, name_c.as_ptr(), $nc_type, 1, self);
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }
                Ok(())
            }
        }
    };
}

impl_putattr!(i8, NC_BYTE, nc_put_att_schar);
impl_putattr!(i16, NC_SHORT, nc_put_att_short);
impl_putattr!(u16, NC_USHORT, nc_put_att_ushort);
impl_putattr!(i32, NC_INT, nc_put_att_int);
impl_putattr!(u32, NC_UINT, nc_put_att_uint);
impl_putattr!(i64, NC_INT64, nc_put_att_longlong);
impl_putattr!(u64, NC_UINT64, nc_put_att_ulonglong);
impl_putattr!(f32, NC_FLOAT, nc_put_att_float);
impl_putattr!(f64, NC_DOUBLE, nc_put_att_double);

impl PutAttr for str {
    fn get_nc_type(&self) -> nc_type {
        NC_CHAR
    }
    fn put(&self, ncid: nc_type, varid: nc_type, name: &str) -> error::Result<()> {
        let name_c: CString = CString::new(name).unwrap();
        let attr_c: CString = CString::new(self).unwrap();
        let err;
        unsafe {
            let _g = LOCK.lock().unwrap();
            err = nc_put_att_text(
                ncid,
                varid,
                name_c.as_ptr(),
                attr_c.to_bytes().len(),
                attr_c.as_ptr(),
            );
        }
        if err != NC_NOERR {
            return Err(err.into());
        }
        Ok(())
    }
}
