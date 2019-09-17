use super::utils::{string_from_c_str, NC_ERRORS};
use super::LOCK;
use netcdf_sys::*;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fmt;

macro_rules! get_attr_as_type {
    ( $me:ident, $nc_type:ident, $rs_type:ty, $nc_fn:ident , $cast:ident ) => {{
        if (!$cast) && ($me.attrtype != $nc_type) {
            return Err("Types are not equivalent and cast==false".to_string());
        }
        let mut err;
        let mut attlen: usize = 0;
        let name_copy: CString = CString::new($me.name.clone()).unwrap();
        unsafe {
            let _g = LOCK.lock().unwrap();
            err = nc_inq_attlen($me.file_id, $me.var_id, name_copy.as_ptr(), &mut attlen);
        }
        if err != NC_NOERR {
            return Err(NC_ERRORS.get(&err).unwrap().clone());
        }
        if attlen != 1 {
            return Err("Multi-value attributes not yet implemented".to_string());
        }
        let mut buf: $rs_type = 0 as $rs_type;
        unsafe {
            let _g = LOCK.lock().unwrap();
            err = $nc_fn($me.file_id, $me.var_id, name_copy.as_ptr(), &mut buf);
        }
        if err != NC_NOERR {
            return Err(NC_ERRORS.get(&err).unwrap().clone());
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
    pub fn get_char(&self, cast: bool) -> Result<String, String> {
        if (!cast) && (self.attrtype != NC_CHAR) {
            return Err("Types are not equivalent and cast==false".to_string());
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
                return Err(NC_ERRORS.get(&err).unwrap().clone());
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
                return Err(NC_ERRORS.get(&err).unwrap().clone());
            }
            let attr_c_str = CStr::from_ptr(attr_char_buf_ptr);
            attr_char_str = string_from_c_str(attr_c_str);
        }
        Ok(attr_char_str)
    }

    pub fn get_byte(&self, cast: bool) -> Result<i8, String> {
        get_attr_as_type!(self, NC_BYTE, i8, nc_get_att_schar, cast)
    }

    pub fn get_short(&self, cast: bool) -> Result<i16, String> {
        get_attr_as_type!(self, NC_SHORT, i16, nc_get_att_short, cast)
    }

    pub fn get_ushort(&self, cast: bool) -> Result<u16, String> {
        get_attr_as_type!(self, NC_USHORT, u16, nc_get_att_ushort, cast)
    }

    pub fn get_int(&self, cast: bool) -> Result<i32, String> {
        get_attr_as_type!(self, NC_INT, i32, nc_get_att_int, cast)
    }

    pub fn get_uint(&self, cast: bool) -> Result<u32, String> {
        get_attr_as_type!(self, NC_UINT, u32, nc_get_att_uint, cast)
    }

    pub fn get_int64(&self, cast: bool) -> Result<i64, String> {
        get_attr_as_type!(self, NC_INT64, i64, nc_get_att_longlong, cast)
    }

    pub fn get_uint64(&self, cast: bool) -> Result<u64, String> {
        get_attr_as_type!(self, NC_UINT64, u64, nc_get_att_ulonglong, cast)
    }

    pub fn get_float(&self, cast: bool) -> Result<f32, String> {
        get_attr_as_type!(self, NC_FLOAT, f32, nc_get_att_float, cast)
    }

    pub fn get_double(&self, cast: bool) -> Result<f64, String> {
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

pub fn init_attributes(
    attrs: &mut HashMap<String, Attribute>,
    file_id: nc_type,
    var_id: nc_type,
    natts_in: nc_type,
) {
    // TODO: better interface to indicate these are var attrs
    let mut nattrs = 0;
    if natts_in == -1 {
        // these are global attrs; have to determine number of attrs
        unsafe {
            let _g = LOCK.lock().unwrap();
            let err = nc_inq_natts(file_id, &mut nattrs);
            assert_eq!(err, NC_NOERR);
        }
    } else {
        nattrs = natts_in;
    }

    // read each attr name, type, value
    let mut attr_type: nc_type = 0;
    for i_attr in 0..nattrs {
        let mut name_buf_vec = vec![0i8; (NC_MAX_NAME + 1) as usize];
        let name_c_str: &CStr;
        unsafe {
            let _g = LOCK.lock().unwrap();
            let name_buf_ptr: *mut i8 = name_buf_vec.as_mut_ptr();
            let err = nc_inq_attname(file_id, var_id, i_attr, name_buf_ptr);
            assert_eq!(err, NC_NOERR);
            let err = nc_inq_atttype(file_id, var_id, name_buf_ptr, &mut attr_type);
            assert_eq!(err, NC_NOERR);
            name_c_str = CStr::from_ptr(name_buf_ptr);
        }
        let name_str: String = string_from_c_str(name_c_str);
        attrs.insert(
            name_str.clone(),
            Attribute {
                name: name_str.clone(),
                attrtype: attr_type,
                id: i_attr,
                var_id: var_id,
                file_id: file_id,
            },
        );
    }
}
