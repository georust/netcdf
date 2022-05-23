#![cfg(feature = "has-mmap")]

use std::os::raw::{c_char, c_int, c_void};

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(all(feature = "has-mmap", feature = "1.6.2"))]
pub struct NC_memio {
    size: usize,
    memory: *mut c_void,
    flags: c_int,
}

extern "C" {
    #[cfg(feature = "has-mmap")]
    pub fn nc_open_mem(
        path: *const c_char,
        mode: c_int,
        size: usize,
        memory: *mut c_void,
        ncidp: *mut c_int,
    ) -> c_int;

    #[cfg(all(feature = "has-mmap", feature = "1.6.2"))]
    pub fn nc_create_mem(
        path: *const c_char,
        mode: c_int,
        initialsize: usize,
        ncidp: *mut c_int,
    ) -> c_int;
    #[cfg(all(feature = "has-mmap", feature = "1.6.2"))]
    pub fn nc_open_memio(
        path: *const c_char,
        mode: c_int,
        info: *mut NC_memio,
        ncidp: *mut c_int,
    ) -> c_int;

    #[cfg(all(feature = "has-mmap", feature = "1.6.2"))]
    pub fn nc_close_memio(ncid: c_int, info: *mut NC_memio) -> c_int;
}
