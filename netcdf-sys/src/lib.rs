#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
#![allow(clippy::type_complexity)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

extern crate hdf5_sys;

#[cfg(feature = "dap")]
extern crate curl_sys;

#[cfg(feature = "static")]
extern crate netcdf_src;

mod consts;
mod functions;
pub use consts::*;
pub use functions::*;

#[cfg(feature = "4.8.0")]
mod dispatch;
#[cfg(feature = "4.8.0")]
pub use dispatch::*;

#[cfg(feature = "has-mmap")]
mod mmap;
#[cfg(feature = "has-mmap")]
pub use mmap::*;

#[cfg(feature = "4.8.0")]
mod filter;
#[cfg(feature = "4.8.0")]
pub use filter::*;

#[cfg(feature = "mpi")]
pub mod par;

/// Global netCDF lock for using all functions in the netCDF library
///
/// Per the NetCDF FAQ: "THE C-BASED LIBRARIES ARE NOT THREAD-SAFE"
/// This lock is the same as the one in `hdf5`, so the two libraries
/// can be used at the same time
pub use hdf5_sys::LOCK as libnetcdf_lock;

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::ffi;
    use std::path;

    #[test]
    fn test_nc_open_close() {
        let mnf_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let test_data_path = path::Path::new(&mnf_dir)
            .join("testdata")
            .join("simple_xy.nc");
        let f = ffi::CString::new(test_data_path.to_str().unwrap()).unwrap();

        let mut ncid: nc_type = -999_999;
        unsafe {
            let _g = libnetcdf_lock.lock();
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
            let _g = libnetcdf_lock.lock();
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
            let _g = libnetcdf_lock.lock();
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
