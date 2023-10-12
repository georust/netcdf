use std::os::raw::{c_char, c_int, c_uint, c_void};

extern "C" {
    pub fn nc_inq_var_filter_ids(
        ncid: c_int,
        varid: c_int,
        nfilters: *mut usize,
        filterids: *mut c_uint,
    ) -> c_int;

    pub fn nc_inq_var_filter_info(
        ncid: c_int,
        varid: c_int,
        id: c_uint,
        nparams: *mut usize,
        params: *mut c_uint,
    ) -> c_int;

    pub fn nc_inq_filter_avail(ncid: c_int, id: c_uint) -> c_int;

}

#[cfg(feature = "4.9.0")]
extern "C" {
    pub fn nc_def_var_bzip2(ncid: c_int, varid: c_int, level: c_int) -> c_int;
    pub fn nc_inq_var_bzip2(
        ncid: c_int,
        varid: c_int,
        hasfilterp: *mut c_int,
        levelp: *mut c_int,
    ) -> c_int;

    pub fn nc_def_var_zstandard(ncid: c_int, varid: c_int, level: c_int) -> c_int;
    pub fn nc_inq_var_zstandard(
        ncid: c_int,
        varid: c_int,
        hasfilterp: *mut c_int,
        levelp: *mut c_int,
    ) -> c_int;

    pub fn nc_def_var_blosc(
        ncid: c_int,
        varid: c_int,
        subcompressor: c_uint,
        level: c_uint,
        blocksize: c_uint,
        addshuffle: c_uint,
    ) -> c_int;
    pub fn nc_inq_var_blosc(
        ncid: c_int,
        varid: c_int,
        hasfilterp: *mut c_int,
        subpcompressorp: *mut c_uint,
        levelp: *mut c_uint,
        blocsizep: *mut c_uint,
        addshufflep: *mut c_uint,
    ) -> c_int;
}

pub const NCZ_CODEC_CLASS_VER: c_int = 1;
pub const NCZ_CODEC_HDF5: c_int = 1;
pub const NCZ_FILTER_DECODE: usize = 0x00000001;

pub type NCZ_get_codec_info_proto = unsafe extern "C" fn() -> *const c_void;

pub type NCZ_codec_info_defaults_proto = unsafe extern "C" fn() -> *const c_void;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct NCZ_codec_t {
    pub version: c_int,
    pub sort: c_int,

    pub codecid: *const c_char,
    pub hdf5id: c_uint,
    pub NCZ_codec_initialize: Option<unsafe extern "C" fn() -> c_void>,
    pub NCZ_codec_finalize: Option<unsafe extern "C" fn() -> c_void>,

    pub NCZ_codec_to_hdf5: Option<
        unsafe extern "C" fn(
            codec: *const c_void,
            nparamsp: *mut usize,
            paramsp: *mut *mut c_uint,
        ) -> c_int,
    >,
    pub NCZ_hdf5_to_codec: Option<
        unsafe extern "C" fn(
            nparams: usize,
            params: *const c_uint,
            codecp: *mut *mut c_char,
        ) -> c_int,
    >,
    pub NCZ_modify_parameters: Option<
        unsafe extern "C" fn(
            ncid: c_int,
            varid: c_int,
            vnparamsp: *mut usize,
            vparamsp: *mut *mut c_uint,
            wnparamsp: *mut usize,
            wparamsp: *mut *mut c_uint,
        ) -> c_int,
    >,
}
