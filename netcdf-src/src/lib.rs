//! Dummy crate for building `netCDF` from source
//!
//! The current pinned version is 4.10.1

#[cfg(feature = "dap")]
extern crate link_cplusplus;

extern crate hdf5_sys;
extern crate libz_sys;
