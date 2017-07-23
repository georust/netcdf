//! Rust bindings for Unidata's [libnetcdf] (http://www.unidata.ucar.edu/software/netcdf/)
//!
//! # Examples
//! 
//! Read:
//! 
//! ```
//! # let path_to_simple_xy = netcdf::test_file("simple_xy.nc");
//! // Open file simple_xy.nc:
//! let file = netcdf::open(&path_to_simple_xy).unwrap();
//!
//! // Access any variable, attribute, or dimension through simple HashMap's:
//! let var = file.root.variables.get("data").unwrap();
//!
//! // Read variable as any NC_TYPE, optionally failing if doing so would
//! // force a cast:
//! let data : Vec<i32> = var.get_int(false).unwrap();
//!
//! // You can also use values() to read the variable, data will be implicitly casted
//! // if needed
//! let data : Vec<i32> = var.values().unwrap();
//!
//! // All variable data is read into 1-dimensional Vec.
//! for x in 0..(6*12) {
//!     assert_eq!(data[x], x as i32);
//! }
//! ```
//!
//! Write:
//!
//! ```
//! let f = netcdf::test_file_new("crabs2.nc"); // just gets a path inside repo
//! 
//! let mut file = netcdf::create(&f).unwrap();
//! 
//! let dim_name = "ncrabs";
//! file.root.add_dimension(dim_name, 10).unwrap();
//! 
//! let var_name = "crab_coolness_level";
//! let data : Vec<i32> = vec![42; 10];
//! // Variable type written to file is inferred from Vec type:
//! file.root.add_variable(
//!             var_name, 
//!             &vec![dim_name.to_string()],
//!             &data
//!         ).unwrap();
//! ```
//!
//! Append:
//!
//! ```
//! // You can also modify a Variable inside an existing netCDF file
//! let f = netcdf::test_file_new("crabs2.nc"); // get the previously written netCDF file path
//! // open it in read/write mode
//! let mut file = netcdf::append(&f).unwrap();
//! // get a mutable binding of the variable "crab_coolness_level"
//! let mut var = file.root.variables.get_mut("crab_coolness_level").unwrap();
//!
//! let data : Vec<i32> = vec![100; 10];
//! // write 5 first elements of the vector `data` into `var` starting at index 2;
//! var.put_values_at(&data, &[2], &[5]);
//! // Change the first value of `var` into '999'
//! var.put_value_at(999 as f32, &[0]);
//! ```

extern crate netcdf_sys;
extern crate ndarray;

#[macro_use]
extern crate lazy_static;
extern crate libc;

use netcdf_sys::{libnetcdf_lock, nc_strerror};
use std::ffi;
use std::str;
use std::path;
use std::env;
use std::fs;
use std::collections::HashMap;

pub mod file;
pub mod variable;
pub mod attribute;
pub mod group;
pub mod dimension;

pub use file::open;
pub use file::create;
pub use file::append;

fn string_from_c_str(c_str: &ffi::CStr) -> String {
    // see http://stackoverflow.com/questions/24145823/rust-ffi-c-string-handling
    // for good rundown
    let buf: &[u8] = c_str.to_bytes();
    let str_slice: &str = str::from_utf8(buf).unwrap();
    str_slice.to_owned()
}


lazy_static! {
    pub static ref NC_ERRORS: HashMap<i32, String> = {
        let mut m = HashMap::new();
        // Invalid error codes are ok; nc_strerror will just return 
        // "Unknown Error"
        for i in -256..256 {
            let msg_cstr : &ffi::CStr;
            unsafe {
                let _g = libnetcdf_lock.lock().unwrap();
                let msg : *const i8 = nc_strerror(i);
                msg_cstr = &ffi::CStr::from_ptr(msg);
            }
            m.insert(i, string_from_c_str(msg_cstr));
        }
        m
    };
}

// Helpers for getting file paths
pub fn test_file(f: &str) -> String {
    let mnf_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = path::Path::new(&mnf_dir).join(
        "testdata").join(f);
    path.to_str().unwrap().to_string()
}

pub fn test_file_new(f: &str) -> String {
    let mnf_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = path::Path::new(&mnf_dir).join("testout");
    let new_file = path.join(f);
    let _err = fs::create_dir(path);
    new_file.to_str().unwrap().to_string()
}
