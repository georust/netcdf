use super::utils::string_from_c_str;
use super::LOCK;
use netcdf_sys::*;
use std::collections::HashMap;
use std::ffi;

#[derive(Clone, Debug)]
pub struct Dimension {
    pub name: String,
    pub len: usize,
    pub id: nc_type,
}

pub fn init_dimensions(dims: &mut HashMap<String, Dimension>, grp_id: i32) {
    // determine number of dims
    let mut ndims = 0i32;
    unsafe {
        let _g = LOCK.lock().unwrap();
        let err = nc_inq_ndims(grp_id, &mut ndims);
        assert_eq!(err, NC_NOERR);
    }

    // read each dim name and length
    for i_dim in 0..ndims {
        let mut buf_vec = vec![0i8; (NC_MAX_NAME + 1) as usize];
        let mut dimlen: usize = 0;
        let c_str: &ffi::CStr;
        unsafe {
            let _g = LOCK.lock().unwrap();
            let buf_ptr: *mut i8 = buf_vec.as_mut_ptr();
            let err = nc_inq_dim(grp_id, i_dim, buf_ptr, &mut dimlen);
            assert_eq!(err, NC_NOERR);
            c_str = ffi::CStr::from_ptr(buf_ptr);
        }
        let str_buf: String = string_from_c_str(c_str);
        dims.insert(
            str_buf.clone(),
            Dimension {
                name: str_buf.clone(),
                len: dimlen as _,
                id: i_dim,
            },
        );
    }
}
