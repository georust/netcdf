#![cfg(feature = "mpi")]
use std::ffi::{c_char, c_int};

use mpi_sys::{MPI_Comm, MPI_Info};

pub const NC_INDEPENDENT: c_int = 0;
pub const NC_COLLECTIVE: c_int = 1;

extern "C" {
    pub fn nc_create_par(
        path: *const c_char,
        cmode: c_int,
        comm: MPI_Comm,
        info: MPI_Info,
        ncidp: *mut c_int,
    ) -> c_int;
    pub fn nc_open_par(
        path: *const c_char,
        mode: c_int,
        comm: MPI_Comm,
        info: MPI_Info,
        ncidp: *mut c_int,
    ) -> c_int;
    pub fn nc_var_par_access(ncid: c_int, varid: c_int, par_access: c_int) -> c_int;
}
