use std::ffi;
use std::collections::HashMap;
use netcdf_sys::*;
use string_from_c_str;

#[derive(Clone)]
pub struct Dimension {
    pub name : String,
    pub len: u64,
    pub id: i32,
}

pub fn init_dimensions(dims: &mut HashMap<String, Dimension>, grp_id: i32) {
    // determine number of dims
    let mut ndims = 0i32;
    unsafe {
        let _g = libnetcdf_lock.lock().unwrap();
        let err = nc_inq_ndims(grp_id, &mut ndims);
        assert_eq!(err, nc_noerr);
    }

    // read each dim name and length
    for i_dim in 0..ndims {
        let mut buf_vec = vec![0i8; (nc_max_name + 1) as usize];
        let mut dimlen : u64 = 0u64;
        let c_str: &ffi::CStr;
        unsafe {
            let _g = libnetcdf_lock.lock().unwrap();
            let buf_ptr : *mut i8 = buf_vec.as_mut_ptr();
            let err = nc_inq_dim(grp_id, i_dim, buf_ptr, &mut dimlen);
            assert_eq!(err, nc_noerr);
            c_str = ffi::CStr::from_ptr(buf_ptr);
        }
        let str_buf: String = string_from_c_str(c_str);
        dims.insert(str_buf.clone(),
                      Dimension{name: str_buf.clone(),
                          len: dimlen,
                          id: i_dim});
    }
}
