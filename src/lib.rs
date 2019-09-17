//! Rust bindings for Unidata's [libnetcdf](http://www.unidata.ucar.edu/software/netcdf/)
//!
//! # Examples
//!
//! Read:
//!
//! ```no_run
//! // Open file simple_xy.nc:
//! let file = netcdf::open("simle_xy.nc").unwrap();
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
//! ```no_run
//! // Write
//! let mut file = netcdf::create("crabs2.nc").unwrap();
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
//! ).unwrap();
//! ```
//!
//! Append:
//! ```no_run
//! // You can also modify a Variable inside an existing netCDF file
//! // open it in read/write mode
//! let mut file = netcdf::append("crabs2.nc").unwrap();
//! // get a mutable binding of the variable "crab_coolness_level"
//! let mut var = file.root.variables.get_mut("crab_coolness_level").unwrap();
//!
//! let data : Vec<i32> = vec![100; 10];
//! // write 5 first elements of the vector `data` into `var` starting at index 2;
//! var.put_values_at(&data, &[2], &[5]);
//! // Change the first value of `var` into '999'
//! var.put_value_at(999 as f32, &[0]);
//! ```

use lazy_static::lazy_static;
use netcdf_sys::{nc_strerror, nc_type};
use std::collections::HashMap;
use std::sync::Mutex;

pub mod attribute;
pub mod dimension;
pub mod file;
pub mod group;
pub mod variable;

pub use attribute::*;
pub use dimension::*;
pub use file::*;
pub use group::*;
pub use variable::*;

pub fn create<P>(name: P) -> Result<File, String>
where
    P: AsRef<std::path::Path>,
{
    File::create(name)
}
pub fn append<P>(name: P) -> Result<File, String>
where
    P: AsRef<std::path::Path>,
{
    File::append(name)
}
pub fn open<P>(name: P) -> Result<File, String>
where
    P: AsRef<std::path::Path>,
{
    File::open(name)
}

lazy_static! {
    pub(crate) static ref LOCK: Mutex<()> = Mutex::new(());
}

pub mod utils {
    use super::*;
    use std::ffi::CStr;

    lazy_static! {
        pub static ref NC_ERRORS: HashMap<nc_type, String> = {
            use std::ffi::CStr;
            let mut m = HashMap::new();
            // Invalid error codes are ok; nc_strerror will just return
            // "Unknown Error"
            for i in -256..256 {
                let msg_cstr : &CStr;
                unsafe {
                    let _g = LOCK.lock().unwrap();
                    let msg : *const i8 = nc_strerror(i);
                    msg_cstr = &CStr::from_ptr(msg);
                }
                m.insert(i, utils::string_from_c_str(msg_cstr));
            }
            m
        };
    }
    pub(crate) fn string_from_c_str(c_str: &CStr) -> String {
        c_str.to_string_lossy().into_owned()
    }
}
