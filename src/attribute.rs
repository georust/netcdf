use super::error;
use super::LOCK;
use netcdf_sys::*;
use std::ffi::{CStr, CString};
use std::fmt;

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
            err = nc_inq_attlen($me.file_id, $me.var_id, name_copy.as_ptr(), &mut attlen);
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
            err = $nc_fn($me.file_id, $me.var_id, name_copy.as_ptr(), &mut buf);
        }
        if err != NC_NOERR {
            return Err(err.into());
        }
        Ok(buf)
    }};
}

#[derive(Debug)]
pub struct Attribute {
    pub name: String,
    pub attrtype: nc_type,
    pub id: nc_type,
    pub var_id: nc_type,
    pub file_id: nc_type,
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
                err = nc_inq_attlen(self.file_id, self.var_id, name_copy.as_ptr(), &mut attlen);
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
                    self.file_id,
                    self.var_id,
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
