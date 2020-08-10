#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

extern crate hdf5_sys;

#[cfg(feature = "dap")]
extern crate curl_sys;

#[cfg(feature = "static")]
extern crate netcdf_src;

mod netcdf_bindings;
mod netcdf_const;
pub use netcdf_bindings::*;
pub use netcdf_const::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::ffi;
    use std::path;

    use lazy_static::lazy_static;
    use std::sync::Mutex;

    // Per the NetCDF FAQ, "THE C-BASED LIBRARIES ARE NOT THREAD-SAFE"
    // So, here is our global mutex.
    // Use lazy-static dependency to avoid use of static_mutex feature which
    // breaks compatibility with stable channel.
    lazy_static! {
        pub static ref libnetcdf_lock: Mutex<()> = Mutex::new(());
    }

    #[test]
    fn test_nc_open_close() {
        let mnf_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let test_data_path = path::Path::new(&mnf_dir)
            .join("testdata")
            .join("simple_xy.nc");
        let f = ffi::CString::new(test_data_path.to_str().unwrap()).unwrap();

        let mut ncid: nc_type = -999_999;
        unsafe {
            let _g = libnetcdf_lock.lock().unwrap();
            let err = nc_open(f.as_ptr(), NC_NOWRITE, &mut ncid);
            assert_eq!(err, NC_NOERR);
            let err = nc_close(ncid);
            assert_eq!(err, NC_NOERR);
        }
    }

    #[test]
    fn test_inq_varid() {
        let mnf_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let test_data_path = path::Path::new(&mnf_dir)
            .join("testdata")
            .join("simple_xy.nc");
        let f = ffi::CString::new(test_data_path.to_str().unwrap()).unwrap();
        let varname = ffi::CString::new("data").unwrap();

        let mut ncid: nc_type = -999_999;
        let mut varid: nc_type = -999_999;
        let mut nvars: nc_type = -999_999;
        unsafe {
            let _g = libnetcdf_lock.lock().unwrap();
            let err = nc_open(f.as_ptr(), NC_NOWRITE, &mut ncid);
            assert_eq!(err, NC_NOERR);
            let err = nc_inq_nvars(ncid, &mut nvars);
            assert_eq!(err, NC_NOERR);
            assert_eq!(nvars, 1);
            let err = nc_inq_varid(ncid, varname.as_ptr(), &mut varid);
            assert_eq!(err, NC_NOERR);
            let err = nc_close(ncid);
            assert_eq!(err, NC_NOERR);
        }
    }

    #[test]
    fn test_get_var() {
        let mnf_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let test_data_path = path::Path::new(&mnf_dir)
            .join("testdata")
            .join("simple_xy.nc");
        let f = ffi::CString::new(test_data_path.to_str().unwrap()).unwrap();
        let varname = ffi::CString::new("data").unwrap();

        let mut ncid: nc_type = -999_999;
        let mut varid: nc_type = -999_999;
        let mut buf: Vec<nc_type> = vec![0; 6 * 12];
        unsafe {
            let _g = libnetcdf_lock.lock().unwrap();
            let err = nc_open(f.as_ptr(), NC_NOWRITE, &mut ncid);
            assert_eq!(err, NC_NOERR);

            let err = nc_inq_varid(ncid, varname.as_ptr(), &mut varid);
            assert_eq!(err, NC_NOERR);

            let err = nc_get_var_int(ncid, varid, buf.as_mut_ptr());
            assert_eq!(err, NC_NOERR);

            let err = nc_close(ncid);
            assert_eq!(err, NC_NOERR);
        }

        for (x, d) in buf.into_iter().enumerate() {
            assert_eq!(d, x as _);
        }
    }
}
