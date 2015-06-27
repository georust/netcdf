#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

#[macro_use]
extern crate lazy_static;

extern crate libc;
use std::sync::Mutex;

include!(concat!(env!("OUT_DIR"), "/netcdf_bindings.rs"));

extern "C" {
    pub static nc_nat: ::libc::c_int;
    pub static nc_byte: ::libc::c_int;
    pub static nc_char: ::libc::c_int;
    pub static nc_short: ::libc::c_int;
    pub static nc_int: ::libc::c_int;
    pub static nc_long: ::libc::c_int;
    pub static nc_float: ::libc::c_int;
    pub static nc_double: ::libc::c_int;
    pub static nc_ubyte: ::libc::c_int;
    pub static nc_ushort: ::libc::c_int;
    pub static nc_uint: ::libc::c_int;
    pub static nc_int64: ::libc::c_int;
    pub static nc_uint64: ::libc::c_int;
    pub static nc_string: ::libc::c_int;
    pub static nc_max_atomic_type: ::libc::c_int;
    pub static nc_vlen	: ::libc::c_int;
    pub static nc_opaque	: ::libc::c_int;
    pub static nc_enum	: ::libc::c_int;
    pub static nc_compound	: ::libc::c_int;
    pub static nc_nowrite	: ::libc::c_int;
    pub static nc_write    	: ::libc::c_int;
    pub static nc_clobber	: ::libc::c_int;
    pub static nc_noclobber    : ::libc::c_int;
    pub static nc_diskless     : ::libc::c_int;
    pub static nc_mmap         : ::libc::c_int;
    pub static nc_classic_model: ::libc::c_int;
    pub static nc_64bit_offset : ::libc::c_int;
    pub static nc_lock         : ::libc::c_int;
    pub static nc_share        : ::libc::c_int;
    pub static nc_netcdf4      : ::libc::c_int;
    pub static nc_mpiio        : ::libc::c_int;
    pub static nc_mpiposix     : ::libc::c_int;
    pub static nc_pnetcdf      : ::libc::c_int;
    pub static nc_format_classic: ::libc::c_int;
    pub static nc_format_64bit  : ::libc::c_int;
    pub static nc_format_netcdf4: ::libc::c_int;
    pub static nc_format_netcdf4_classic : ::libc::c_int;
    pub static nc_format_nc3    : ::libc::c_int;
    pub static nc_format_nc_hdf5: ::libc::c_int;
    pub static nc_format_nc_hdf4: ::libc::c_int;
    pub static nc_format_pnetcdf: ::libc::c_int;
    pub static nc_format_dap2   : ::libc::c_int;
    pub static nc_format_dap4   : ::libc::c_int;
    pub static nc_format_undefined: ::libc::c_int;
    pub static nc_sizehint_default : ::libc::c_int;
    pub static nc_global : ::libc::c_int;
    pub static nc_max_dims	: ::libc::c_int;
    pub static nc_max_attrs	: ::libc::c_int;
    pub static nc_max_vars	: ::libc::c_int;
    pub static nc_max_name	: ::libc::c_int;
    pub static nc_max_var_dims	: ::libc::c_int;
    pub static nc_max_hdf4_name : ::libc::c_int;
    pub static nc_endian_native : ::libc::c_int;
    pub static nc_endian_little : ::libc::c_int;
    pub static nc_endian_big    : ::libc::c_int;
    pub static nc_chunked    : ::libc::c_int;
    pub static nc_contiguous : ::libc::c_int;
    pub static nc_nochecksum : ::libc::c_int;
    pub static nc_fletcher32 : ::libc::c_int;
    pub static nc_noshuffle : ::libc::c_int;
    pub static nc_shuffle   : ::libc::c_int;
    pub static nc_noerr	: ::libc::c_int;
    pub static nc2_err         : ::libc::c_int;
    pub static nc_ebadid	: ::libc::c_int;
    pub static nc_enfile	: ::libc::c_int;
    pub static nc_eexist	: ::libc::c_int;
    pub static nc_einval	: ::libc::c_int;
    pub static nc_eperm	: ::libc::c_int;
    pub static nc_enotindefine	: ::libc::c_int;
    pub static nc_eindefine	: ::libc::c_int;
    pub static nc_einvalcoords	: ::libc::c_int;
    pub static nc_emaxdims	: ::libc::c_int;
    pub static nc_enameinuse	: ::libc::c_int;
    pub static nc_enotatt	: ::libc::c_int;
    pub static nc_emaxatts	: ::libc::c_int;
    pub static nc_ebadtype	: ::libc::c_int;
    pub static nc_ebaddim	: ::libc::c_int;
    pub static nc_eunlimpos	: ::libc::c_int;
    pub static nc_emaxvars	: ::libc::c_int;
    pub static nc_enotvar	: ::libc::c_int;
    pub static nc_eglobal	: ::libc::c_int;
    pub static nc_enotnc	: ::libc::c_int;
    pub static nc_ests        	: ::libc::c_int;
    pub static nc_emaxname    	: ::libc::c_int;
    pub static nc_eunlimit    	: ::libc::c_int;
    pub static nc_enorecvars  	: ::libc::c_int;
    pub static nc_echar	: ::libc::c_int;
    pub static nc_eedge	: ::libc::c_int;
    pub static nc_estride	: ::libc::c_int;
    pub static nc_ebadname	: ::libc::c_int;
    pub static nc_erange	: ::libc::c_int;
    pub static nc_enomem	: ::libc::c_int;
    pub static nc_evarsize     : ::libc::c_int;
    pub static nc_edimsize     : ::libc::c_int;
    pub static nc_etrunc       : ::libc::c_int;
    pub static nc_eaxistype    : ::libc::c_int;
    pub static nc_edap         : ::libc::c_int;
    pub static nc_ecurl        : ::libc::c_int;
    pub static nc_eio          : ::libc::c_int;
    pub static nc_enodata      : ::libc::c_int;
    pub static nc_edapsvc      : ::libc::c_int;
    pub static nc_edas		: ::libc::c_int;
    pub static nc_edds		: ::libc::c_int;
    pub static nc_edatadds	: ::libc::c_int;
    pub static nc_edapurl	: ::libc::c_int;
    pub static nc_edapconstraint : ::libc::c_int;
    pub static nc_etranslation : ::libc::c_int;
    pub static nc_eaccess      : ::libc::c_int;
    pub static nc_eauth        : ::libc::c_int;
    pub static nc_enotfound     : ::libc::c_int;
    pub static nc_ecantremove   : ::libc::c_int;
    pub static nc4_first_error  : ::libc::c_int;
    pub static nc_ehdferr       : ::libc::c_int;
    pub static nc_ecantread     : ::libc::c_int;
    pub static nc_ecantwrite    : ::libc::c_int;
    pub static nc_ecantcreate   : ::libc::c_int;
    pub static nc_efilemeta     : ::libc::c_int;
    pub static nc_edimmeta      : ::libc::c_int;
    pub static nc_eattmeta      : ::libc::c_int;
    pub static nc_evarmeta      : ::libc::c_int;
    pub static nc_enocompound   : ::libc::c_int;
    pub static nc_eattexists    : ::libc::c_int;
    pub static nc_enotnc4       : ::libc::c_int;
    pub static nc_estrictnc3    : ::libc::c_int;
    pub static nc_enotnc3       : ::libc::c_int;
    pub static nc_enopar        : ::libc::c_int;
    pub static nc_eparinit      : ::libc::c_int;
    pub static nc_ebadgrpid     : ::libc::c_int;
    pub static nc_ebadtypid     : ::libc::c_int;
    pub static nc_etypdefined   : ::libc::c_int;
    pub static nc_ebadfield     : ::libc::c_int;
    pub static nc_ebadclass     : ::libc::c_int;
    pub static nc_emaptype      : ::libc::c_int;
    pub static nc_elatefill     : ::libc::c_int;
    pub static nc_elatedef      : ::libc::c_int;
    pub static nc_edimscale     : ::libc::c_int;
    pub static nc_enogrp        : ::libc::c_int;
    pub static nc_estorage      : ::libc::c_int;
    pub static nc_ebadchunk     : ::libc::c_int;
    pub static nc_enotbuilt     : ::libc::c_int;
    pub static nc_ediskless     : ::libc::c_int;
    pub static nc_ecantextend   : ::libc::c_int;
    pub static nc_empi          : ::libc::c_int;
    pub static nc4_last_error   : ::libc::c_int;
    pub static nc_have_new_chunking_api : ::libc::c_int;
    pub static nc_eurl		: ::libc::c_int;
    pub static nc_econstraint  : ::libc::c_int;
    pub static nc_fill  : ::libc::c_int;
    pub static nc_nofill  : ::libc::c_int;

    pub static nc_unlimited: ::libc::c_long;

    pub static nc_fill_byte: i8;
    pub static nc_fill_char: u8;
    pub static nc_fill_short: i16;
    pub static nc_fill_int: i32;
    pub static nc_fill_float: f32;
    pub static nc_fill_double: f64;
    pub static nc_fill_ubyte: u8;
    pub static nc_fill_ushort: u16;
    pub static nc_fill_uint: u32;
    pub static nc_fill_int64: i64;
    pub static nc_fill_uint64: u64;
    pub static nc_fill_string: *const ::libc::c_char;

    pub static nc_align_chunk: ::libc::size_t;
}

// NetCDF types map well to Rust types, for for completeness, here are 
// definitions of min/max constants:
pub const nc_max_byte : i8 = std::i8::MAX;
pub const nc_min_byte : i8 = std::i8::MIN;
pub const nc_max_char : u8 = std::u8::MAX;
pub const nc_max_short : i16 = std::i16::MAX;
pub const nc_min_short : i16 = std::i16::MIN;
pub const nc_max_int : i32 = std::i32::MAX;
pub const nc_min_int : i32 = std::i32::MIN;
pub const nc_max_float : f32 = std::f32::MAX;
pub const nc_min_float : f32 = std::f32::MIN;
pub const nc_max_double : f64 = std::f64::MAX;
pub const nc_min_double : f64 = std::f64::MIN;
pub const nc_max_ubyte : u8 = std::u8::MAX;
pub const nc_max_ushort : u16 = std::u16::MAX;
pub const nc_max_uint : u32 = std::u32::MAX;
pub const nc_max_int64 : i64 = std::i64::MAX;
pub const nc_min_int64 : i64 = std::i64::MIN;
pub const nc_max_uint64 : u64 = std::u64::MAX;

// Per the NetCDF FAQ, "THE C-BASED LIBRARIES ARE NOT THREAD-SAFE"
// So, here is our global mutex.
// Use lazy-static dependency to avoid use of static_mutex feature which 
// breaks compatibility with stable channel.
lazy_static! {
    pub static ref libnetcdf_lock: Mutex<()> = Mutex::new(());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path;
    use std::env;
    use std::ffi;

    #[test]
    fn test_nc_open_close() {
        let mnf_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let test_data_path = path::Path::new(&mnf_dir).join(
            "testdata").join("simple_xy.nc");
        let f = ffi::CString::new(test_data_path.to_str().unwrap()).unwrap();
        
        let mut ncid : i32 = -999999i32;
        unsafe {
            let _g = libnetcdf_lock.lock().unwrap();
            let err = nc_open(f.as_ptr(), nc_nowrite, &mut ncid);
            assert_eq!(err, nc_noerr);
            let err = nc_close(ncid);
            assert_eq!(err, nc_noerr);
        }
    }

    #[test]
    fn test_inq_varid() {
        let mnf_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let test_data_path = path::Path::new(&mnf_dir).join(
            "testdata").join("simple_xy.nc");
        let f = ffi::CString::new(test_data_path.to_str().unwrap()).unwrap();
        let varname = ffi::CString::new("data").unwrap();
        
        let mut ncid : i32 = -999999i32;
        let mut varid : i32 = -999999i32;
        let mut nvars : i32 = -999999i32;
        unsafe {
            let _g = libnetcdf_lock.lock().unwrap();
            let err = nc_open(f.as_ptr(), nc_nowrite, &mut ncid);
            assert_eq!(err, nc_noerr);
            let err = nc_inq_nvars(ncid, &mut nvars);
            assert_eq!(err, nc_noerr);
            assert_eq!(nvars, 1);
            let err = nc_inq_varid(ncid, varname.as_ptr(), &mut varid);
            assert_eq!(err, nc_noerr);
            let err = nc_close(ncid);
            assert_eq!(err, nc_noerr);
        }
    }

    #[test]
    fn test_get_var() {
        let mnf_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let test_data_path = path::Path::new(&mnf_dir).join(
            "testdata").join("simple_xy.nc");
        let f = ffi::CString::new(test_data_path.to_str().unwrap()).unwrap();
        let varname = ffi::CString::new("data").unwrap();
        
        let mut ncid : i32 = -999999i32;
        let mut varid : i32 = -999999i32;
        let mut buf : Vec<i32> = Vec::with_capacity(6*12);
        unsafe {
            buf.set_len(6*12);

            let _g = libnetcdf_lock.lock().unwrap();
            let err = nc_open(f.as_ptr(), nc_nowrite, &mut ncid);
            assert_eq!(err, nc_noerr);

            let err = nc_inq_varid(ncid, varname.as_ptr(), &mut varid);
            assert_eq!(err, nc_noerr);

            let err = nc_get_var_int(ncid, varid, buf.as_mut_ptr());
            assert_eq!(err, nc_noerr);

            let err = nc_close(ncid);
            assert_eq!(err, nc_noerr);
        }

        for x in 0..(6*12) {
            assert_eq!(buf[x], x as i32);
        }
    }

}
