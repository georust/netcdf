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
//! let var = file.root().variables().get("data").unwrap();
//!
//! // Read variable as any NC_TYPE
//! let data : i32 = var.get_value::<i32>(None).unwrap();
//!
//! // You can also use values() to read the variable, data will be implicitly casted
//! // if needed. Pass None where you don't care about the hyperslab
//! let data  = var.get_values::<i32>(None, None).unwrap();
//!
//! // All variable data is read into an ndarray
//! println!("{}", data);
//! ```
//!
//! Write:
//!
//! ```no_run
//! // Write
//! let mut file = netcdf::create("crabs2.nc").unwrap();
//!
//! let dim_name = "ncrabs";
//! file.root_mut().add_dimension(dim_name, 10).unwrap();
//!
//! let var_name = "crab_coolness_level";
//! let data : Vec<i32> = vec![42; 10];
//! // Variable type written to file
//! let var = file.root_mut().add_variable::<i32>(
//!             var_name,
//!             &vec![dim_name],
//! ).unwrap();
//! var.put_values(&data, None, None);
//! ```
//!
//! Append:
//! ```no_run
//! // You can also modify a Variable inside an existing netCDF file
//! // open it in read/write mode
//! let mut file = netcdf::append("crabs2.nc").unwrap();
//! // get a mutable binding of the variable "crab_coolness_level"
//! let mut var = file.root_mut().variable_mut("crab_coolness_level").unwrap();
//!
//! let data : Vec<i32> = vec![100; 10];
//! // write 5 first elements of the vector `data` into `var` starting at index 2;
//! var.put_values(&data, Some(&[2]), Some(&[5]));
//! // Change the first value of `var` into '999'
//! var.put_value(999.0f32, Some(&[0]));
//! ```

use lazy_static::lazy_static;
use netcdf_sys::nc_type;
use std::sync::Mutex;

pub mod attribute;
pub mod dimension;
pub mod error;
pub mod file;
pub mod group;
pub mod variable;

pub use attribute::*;
pub use dimension::*;
pub use file::*;
pub use group::*;
pub use variable::*;

/// Open a netcdf file in create mode
pub fn create<P>(name: P) -> error::Result<File>
where
    P: AsRef<std::path::Path>,
{
    File::create(name)
}

/// Open a netcdf file in append mode
pub fn append<P>(name: P) -> error::Result<File>
where
    P: AsRef<std::path::Path>,
{
    File::append(name)
}

/// Open a netcdf file in read mode
pub fn open<P>(name: P) -> error::Result<File>
where
    P: AsRef<std::path::Path>,
{
    File::open(name)
}

lazy_static! {
    /// Use this when accessing netcdf functions
    pub(crate) static ref LOCK: Mutex<()> = Mutex::new(());
}
