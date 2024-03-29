use crate::{
    error::{checked, Result},
    with_lock,
};

use netcdf_sys::nc_type;

#[derive(Copy, Clone)]
#[repr(i32)]
pub(crate) enum AccessMode {
    Independent = netcdf_sys::par::NC_INDEPENDENT,
    Collective = netcdf_sys::par::NC_COLLECTIVE,
}

pub(crate) fn set_access_mode(ncid: nc_type, varid: nc_type, mode: AccessMode) -> Result<()> {
    checked(with_lock(|| unsafe {
        netcdf_sys::par::nc_var_par_access(ncid, varid, mode as i32 as std::ffi::c_int)
    }))
}
