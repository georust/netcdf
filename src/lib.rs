//! Rust bindings for Unidata's [libnetcdf](http://www.unidata.ucar.edu/software/netcdf/)
//!
//! # Examples
//!
//! Read:
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Open file simple_xy.nc:
//! let file = netcdf::open("simle_xy.nc")?;
//!
//! // Access any variable, attribute, or dimension through lookups on hashmaps
//! let var = &file.variable("data").expect("Could not find variable 'data'");
//!
//! // Read variable as numeric types
//! let data_i32 = var.value::<i32>(None)?;
//! let data_f32 : f32 = var.value(None)?;
//!
//! // You can also use values() to read the variable, data will be read as the type given as type parameter (in this case T=i32)
//! // Pass (None, None) when you don't care about the hyperslab indexes (get all data)
//! # #[cfg(feature = "ndarray")]
//! let data = var.values::<i32>(None, None)?;
//! # Ok(()) }
//! ```
//!
//! Write:
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Write
//! let mut file = netcdf::create("crabs2.nc")?;
//!
//! let dim_name = "ncrabs";
//! file.add_dimension(dim_name, 10)?;
//!
//! let var_name = "crab_coolness_level";
//! let data : Vec<i32> = vec![42; 10];
//! // Variable type written to file
//! let mut var = file.add_variable::<i32>(
//!             var_name,
//!             &[dim_name],
//! )?;
//! var.put_values(&data, None, None);
//! # Ok(()) }
//! ```
//!
//! Append:
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // You can also modify a Variable inside an existing netCDF file
//! // open it in read/write mode
//! let mut file = netcdf::append("crabs2.nc")?;
//! // get a mutable binding of the variable "crab_coolness_level"
//! let mut var = file.variable_mut("crab_coolness_level").unwrap();
//!
//! let data : Vec<i32> = vec![100; 10];
//! // write 5 first elements of the vector `data` into `var` starting at index 2;
//! var.put_values(&data[..5], Some(&[2]), Some(&[5]));
//! // Change the first value of `var` into '999'
//! var.put_value(999.0f32, Some(&[0]));
//! # Ok(()) }
//! ```

#![warn(missing_docs)]
#![allow(clippy::must_use_candidate)]

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
///
/// Will overwrite exising file
pub fn create<P>(name: P) -> error::Result<MutableFile>
where
    P: AsRef<std::path::Path>,
{
    RawFile::create(name.as_ref())
}

/// Open a netcdf file in append mode
pub fn append<P>(name: P) -> error::Result<MutableFile>
where
    P: AsRef<std::path::Path>,
{
    RawFile::append(name.as_ref())
}

/// Open a netcdf file in read mode
pub fn open<P>(name: P) -> error::Result<File>
where
    P: AsRef<std::path::Path>,
{
    RawFile::open(name.as_ref())
}

#[cfg(feature = "memory")]
/// Open a netcdf file from a buffer
pub fn open_mem<'a>(name: Option<&str>, mem: &'a [u8]) -> error::Result<MemFile<'a>> {
    RawFile::open_from_memory(name, mem)
}

lazy_static! {
    /// Use this when accessing netcdf functions
    pub(crate) static ref LOCK: Mutex<()> = Mutex::new(());
}

/// All functions should be wrapped in this locker. Disregarding this, expect
/// segfaults, especially on non-threadsafe hdf5 builds
pub(crate) fn with_lock<F: FnMut() -> nc_type>(mut f: F) -> nc_type {
    let _l = LOCK.lock().unwrap();
    f()
}

pub(crate) mod utils {
    use super::*;
    use netcdf_sys::{NC_EMAXNAME, NC_MAX_NAME};
    /// Use this function for short netcdf names to avoid the allocation
    /// for a `CString`
    pub(crate) fn short_name_to_bytes(name: &str) -> error::Result<[u8; NC_MAX_NAME as usize + 1]> {
        if name.len() > NC_MAX_NAME as _ {
            Err(NC_EMAXNAME.into())
        } else {
            let len = name
                .bytes()
                .position(|x| x == 0)
                .unwrap_or_else(|| name.len());
            let mut bytes = [0_u8; NC_MAX_NAME as usize + 1];
            bytes[..len].copy_from_slice(name.as_bytes());
            Ok(bytes)
        }
    }
}
