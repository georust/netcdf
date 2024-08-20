#![allow(clippy::unreadable_literal)]
#![allow(non_snake_case)]
#![allow(clippy::excessive_precision)]

use ::std::os::raw::c_int;

use super::nc_type;

pub const NC_NAT: nc_type = 0;
pub const NC_BYTE: nc_type = 1;
pub const NC_CHAR: nc_type = 2;
pub const NC_SHORT: nc_type = 3;
pub const NC_INT: nc_type = 4;
pub const NC_LONG: nc_type = 4;
pub const NC_FLOAT: nc_type = 5;
pub const NC_DOUBLE: nc_type = 6;
pub const NC_UBYTE: nc_type = 7;
pub const NC_USHORT: nc_type = 8;
pub const NC_UINT: nc_type = 9;
pub const NC_INT64: nc_type = 10;
pub const NC_UINT64: nc_type = 11;
pub const NC_STRING: nc_type = 12;

pub const NC_MAX_ATOMIC_TYPE: nc_type = 12;

pub const NC_VLEN: nc_type = 13;
pub const NC_OPAQUE: nc_type = 14;
pub const NC_ENUM: nc_type = 15;
pub const NC_COMPOUND: nc_type = 16;

pub const NC_FIRSTUSERTYPEID: nc_type = 32;

pub const NC_FILL_BYTE: i8 = -127;
pub const NC_FILL_CHAR: u8 = 0;
pub const NC_FILL_SHORT: i16 = -32767;
pub const NC_FILL_INT: i32 = -2147483647;
pub const NC_FILL_FLOAT: f32 = 9.9692099683868690e+36;
pub const NC_FILL_DOUBLE: f64 = 9.9692099683868690e+36;
pub const NC_FILL_UBYTE: u8 = 255;
pub const NC_FILL_USHORT: u16 = 65535;
pub const NC_FILL_UINT: u32 = 4294967295;
pub const NC_FILL_INT64: i64 = -9223372036854775806;
pub const NC_FILL_UINT64: u64 = 18446744073709551614;
pub const NC_FILL_STRING: &[u8] = b"\0";

pub const NC_MAX_BYTE: i8 = 127;
pub const NC_MIN_BYTE: i8 = -NC_MAX_BYTE - 1;
pub const NC_MAX_CHAR: u8 = 255;
pub const NC_MAX_SHORT: i16 = 32767;
pub const NC_MIN_SHORT: i16 = -NC_MAX_SHORT - 1;
pub const NC_MAX_INT: i32 = 2147483647;
pub const NC_MIN_INT: i32 = -NC_MAX_INT - 1;
pub const NC_MAX_FLOAT: f32 = 3.402823466e+38;
pub const NC_MIN_FLOAT: f32 = -NC_MAX_FLOAT;
pub const NC_MAX_DOUBLE: f64 = 1.7976931348623157e+308;
pub const NC_MIN_DOUBLE: f64 = -NC_MAX_DOUBLE;
pub const NC_MAX_UBYTE: u8 = 255;
pub const NC_MAX_USHORT: u16 = 65535;
pub const NC_MAX_UINT: u32 = 4294967295;
pub const NC_MAX_INT64: i64 = 9223372036854775807;
pub const NC_MIN_INT64: i64 = -9223372036854775808;
pub const NC_MAX_UINT64: u64 = 18446744073709551615;

pub const _FillValue: &[u8] = b"_FillValue\0";

pub const NC_FILL: c_int = 0x000;
pub const NC_NOFILL: c_int = 0x100;

pub const NC_NOWRITE: c_int = 0x0000;
pub const NC_WRITE: c_int = 0x0001;

pub const NC_CLOBBER: c_int = 0x0000;
pub const NC_NOCLOBBER: c_int = 0x0004;

pub const NC_DISKLESS: c_int = 0x0008;
pub const NC_MMAP: c_int = 0x0010;

#[cfg(feature = "4.4.0")]
pub const NC_64BIT_DATA: c_int = 0x0020;
#[cfg(feature = "4.4.0")]
pub const NC_CDF5: c_int = NC_64BIT_DATA;

#[cfg(feature = "4.6.2")]
pub const NC_UDF0: c_int = 0x0040;
#[cfg(feature = "4.6.2")]
pub const NC_UDF1: c_int = 0x0080;

pub const NC_CLASSIC_MODEL: c_int = 0x0100;
pub const NC_64BIT_OFFSET: c_int = 0x0200;

pub const NC_LOCK: c_int = 0x0400;

pub const NC_SHARE: c_int = 0x0800;

pub const NC_NETCDF4: c_int = 0x1000;

#[cfg_attr(
    feature = "4.6.2",
    deprecated(note = "Parallel I/O is initiated by calling nc_create_par and nc_open_par")
)]
pub const NC_MPIIO: c_int = 0x2000;
#[cfg_attr(
    feature = "4.6.2",
    deprecated(note = "Parallel I/O is initiated by calling nc_create_par and nc_open_par"),
    allow(deprecated)
)]
pub const NC_PNETCDF: c_int = NC_MPIIO;
#[cfg_attr(
    feature = "4.6.2",
    deprecated(note = "Parallel I/O is initiated by calling nc_create_par and nc_open_par")
)]
pub const NC_MPIPOSIX: c_int = {
    #[cfg(feature = "4.6.2")]
    {
        0x4000
    }
    #[cfg(not(feature = "4.6.2"))]
    {
        NC_MPIIO
    }
};

#[cfg(feature = "4.6.2")]
pub const NC_PERSIST: c_int = 0x4000;
pub const NC_INMEMORY: c_int = 0x8000;

#[cfg(feature = "4.9.0")]
pub const NC_NOATTCREORD: c_int = 0x20000;
#[cfg(feature = "4.9.0")]
pub const NC_NODIMSCALE_ATTACH: c_int = 0x40000;

#[cfg(feature = "4.6.2")]
pub const NC_MAX_MAGIC_NUMBER_LEN: usize = 8;

pub const NC_FORMAT_CLASSIC: c_int = 1;
pub const NC_FORMAT_64BIT_OFFSET: c_int = 2;
pub const NC_FORMAT_64BIT: c_int = NC_FORMAT_64BIT_OFFSET;
pub const NC_FORMAT_NETCDF4: c_int = 3;
pub const NC_FORMAT_NETCDF4_CLASSIC: c_int = 4;
pub const NC_FORMAT_64BIT_DATA: c_int = 5;
pub const NC_FORMAT_CDF5: c_int = NC_FORMAT_64BIT_DATA;

#[cfg(feature = "4.7.0")]
pub const NC_FORMAT_ALL: c_int =
    NC_64BIT_OFFSET | NC_64BIT_DATA | NC_CLASSIC_MODEL | NC_NETCDF4 | NC_UDF0 | NC_UDF1;

#[deprecated(note = "Use NC_FORMATX_NC3")]
pub const NC_FORMAT_NC3: c_int = NC_FORMATX_NC3;
#[deprecated(note = "Use NC_FORMATX_HDF5")]
pub const NC_FORMAT_NC_HDF5: c_int = NC_FORMATX_NC_HDF5;
#[deprecated(note = "Use NC_FORMATX_NC4")]
pub const NC_FORMAT_NC4: c_int = NC_FORMATX_NC4;
#[deprecated(note = "Use NC_FORMATX_HDF4")]
pub const NC_FORMAT_NC_HDF4: c_int = NC_FORMATX_NC_HDF4;
#[deprecated(note = "Use NC_FORMATX_PNETCDF")]
pub const NC_FORMAT_PNETCDF: c_int = NC_FORMATX_PNETCDF;
#[deprecated(note = "Use NC_FORMATX_DAP2")]
pub const NC_FORMAT_DAP2: c_int = NC_FORMATX_DAP2;
#[deprecated(note = "Use NC_FORMATX_DAP4")]
pub const NC_FORMAT_DAP4: c_int = NC_FORMATX_DAP4;
#[deprecated(note = "Use NC_FORMATX_UNDEFINED")]
pub const NC_FORMAT_UNDEFINED: c_int = NC_FORMATX_UNDEFINED;

pub const NC_FORMATX_NC3: c_int = 1;
pub const NC_FORMATX_NC_HDF5: c_int = 2;
pub const NC_FORMATX_NC4: c_int = NC_FORMATX_NC_HDF5;
pub const NC_FORMATX_NC_HDF4: c_int = 3;
pub const NC_FORMATX_PNETCDF: c_int = 4;
pub const NC_FORMATX_DAP2: c_int = 5;
pub const NC_FORMATX_DAP4: c_int = 6;
#[cfg(feature = "4.6.2")]
pub const NC_FORMATX_UDF0: c_int = 8;
#[cfg(feature = "4.6.2")]
pub const NC_FORMATX_UDF1: c_int = 9;
#[cfg(feature = "4.7.0")]
pub const NC_FORMATX_NCZARR: c_int = 10;
pub const NC_FORMATX_UNDEFINED: c_int = 0;

pub const NC_SIZEHINT_DEFAULT: c_int = 0;

pub const NC_ALIGN_CHUNK: usize = !0;

pub const NC_UNLIMITED: usize = 0;

pub const NC_GLOBAL: c_int = -1;

#[cfg_attr(feature = "4.5.0", deprecated(note = "Not enforced"))]
pub const NC_MAX_DIMS: c_int = 1024;
#[cfg_attr(feature = "4.5.0", deprecated(note = "Not enforced"))]
pub const NC_MAX_ATTRS: c_int = 8192;
#[cfg_attr(feature = "4.5.0", deprecated(note = "Not enforced"))]
pub const NC_MAX_VARS: c_int = 8192;
pub const NC_MAX_NAME: c_int = 256;
pub const NC_MAX_VAR_DIMS: c_int = 1024;

pub const NC_MAX_HDF4_NAME: c_int = NC_MAX_NAME;

pub const NC_ENDIAN_NATIVE: c_int = 0;
pub const NC_ENDIAN_LITTLE: c_int = 1;
pub const NC_ENDIAN_BIG: c_int = 2;

pub const NC_CHUNKED: c_int = 0;
pub const NC_CONTIGUOUS: c_int = 1;
#[cfg(feature = "4.7.4")]
pub const NC_COMPACT: c_int = 2;
#[cfg(feature = "4.8.1")]
pub const NC_UNKNOWN_STORAGE: c_int = 3;
#[cfg(feature = "4.8.1")]
pub const NC_VIRTUAL: c_int = 4;

pub const NC_NOCHECKSUM: c_int = 0;
pub const NC_FLETCHER32: c_int = 1;

pub const NC_NOSHUFFLE: c_int = 0;
pub const NC_SHUFFLE: c_int = 1;

#[cfg(feature = "4.6.0")]
pub const NC_MIN_DEFLATE_LEVEL: c_int = 0;
#[cfg(feature = "4.6.0")]
pub const NC_MAX_DEFLATE_LEVEL: c_int = 9;

pub const fn NC_ISSYSERR(err: c_int) -> bool {
    err > 0
}

pub const NC_NOERR: c_int = 0;
pub const NC2_ERR: c_int = -1;

pub const NC_EBADID: c_int = -33;
pub const NC_ENFILE: c_int = -34;
pub const NC_EEXIST: c_int = -35;
pub const NC_EINVAL: c_int = -36;
pub const NC_EPERM: c_int = -37;
pub const NC_ENOTINDEFINE: c_int = -38;
pub const NC_EINDEFINE: c_int = -39;
pub const NC_EINVALCOORDS: c_int = -40;
pub const NC_EMAXDIMS: c_int = -41;
pub const NC_ENAMEINUSE: c_int = -42;
pub const NC_ENOTATT: c_int = -43;
pub const NC_EMAXATTS: c_int = -44;
pub const NC_EBADTYPE: c_int = -45;
pub const NC_EBADDIM: c_int = -46;
pub const NC_EUNLIMPOS: c_int = -47;
pub const NC_EMAXVARS: c_int = -48;
pub const NC_ENOTVAR: c_int = -49;
pub const NC_EGLOBAL: c_int = -50;
pub const NC_ENOTNC: c_int = -51;
pub const NC_ESTS: c_int = -52;
pub const NC_EMAXNAME: c_int = -53;
pub const NC_EUNLIMIT: c_int = -54;
pub const NC_ENORECVARS: c_int = -55;
pub const NC_ECHAR: c_int = -56;
pub const NC_EEDGE: c_int = -57;
pub const NC_ESTRIDE: c_int = -58;
pub const NC_EBADNAME: c_int = -59;
pub const NC_ERANGE: c_int = -60;
pub const NC_ENOMEM: c_int = -61;
pub const NC_EVARSIZE: c_int = -62;
pub const NC_EDIMSIZE: c_int = -63;
pub const NC_ETRUNC: c_int = -64;
pub const NC_EAXISTYPE: c_int = -65;
pub const NC_EDAP: c_int = -66;
pub const NC_ECURL: c_int = -67;
pub const NC_EIO: c_int = -68;
pub const NC_ENODATA: c_int = -69;
pub const NC_EDAPSVC: c_int = -70;
pub const NC_EDAS: c_int = -71;
pub const NC_EDDS: c_int = -72;
pub const NC_EDMR: c_int = NC_EDDS;
pub const NC_EDATADDS: c_int = -73;
pub const NC_EDATADAP: c_int = NC_EDATADDS;
pub const NC_EDAPURL: c_int = -74;
pub const NC_EDAPCONSTRAINT: c_int = -75;
pub const NC_ETRANSLATION: c_int = -76;
pub const NC_EACCESS: c_int = -77;
pub const NC_EAUTH: c_int = -78;
pub const NC_ENOTFOUND: c_int = -90;
pub const NC_ECANTREMOVE: c_int = -91;
#[cfg(feature = "4.6.1")]
pub const NC_EINTERNAL: c_int = -92;
#[cfg(feature = "4.6.2")]
pub const NC_EPNETCDF: c_int = -93;
pub const NC4_FIRST_ERROR: c_int = -100;
pub const NC_EHDFERR: c_int = -101;
pub const NC_ECANTREAD: c_int = -102;
pub const NC_ECANTWRITE: c_int = -103;
pub const NC_ECANTCREATE: c_int = -104;
pub const NC_EFILEMETA: c_int = -105;
pub const NC_EDIMMETA: c_int = -106;
pub const NC_EATTMETA: c_int = -107;
pub const NC_EVARMETA: c_int = -108;
pub const NC_ENOCOMPOUND: c_int = -109;
pub const NC_EATTEXISTS: c_int = -110;
pub const NC_ENOTNC4: c_int = -111;
pub const NC_ESTRICTNC3: c_int = -112;
pub const NC_ENOTNC3: c_int = -113;
pub const NC_ENOPAR: c_int = -114;
pub const NC_EPARINIT: c_int = -115;
pub const NC_EBADGRPID: c_int = -116;
pub const NC_EBADTYPID: c_int = -117;
pub const NC_ETYPDEFINED: c_int = -118;
pub const NC_EBADFIELD: c_int = -119;
pub const NC_EBADCLASS: c_int = -120;
pub const NC_EMAPTYPE: c_int = -121;
pub const NC_ELATEFILL: c_int = -122;
pub const NC_ELATEDEF: c_int = -123;
pub const NC_EDIMSCALE: c_int = -124;
pub const NC_ENOGRP: c_int = -125;
pub const NC_ESTORAGE: c_int = -126;
pub const NC_EBADCHUNK: c_int = -127;
pub const NC_ENOTBUILT: c_int = -128;
pub const NC_EDISKLESS: c_int = -129;
pub const NC_ECANTEXTEND: c_int = -130;
pub const NC_EMPI: c_int = -131;
#[cfg(feature = "4.6.0")]
pub const NC_EFILTER: c_int = -132;
#[cfg(feature = "4.6.0")]
pub const NC_ERCFILE: c_int = -133;
#[cfg(feature = "4.6.0")]
pub const NC_NULLPAD: c_int = -134;
#[cfg(feature = "4.6.2")]
pub const NC_EINMEMORY: c_int = -135;
#[cfg(feature = "4.7.4")]
pub const NC_ENOFILTER: c_int = -136;
#[cfg(feature = "4.8.0")]
pub const NC_ENCZARR: c_int = -137;
#[cfg(feature = "4.8.0")]
pub const NC_ES3: c_int = -138;
#[cfg(feature = "4.8.0")]
pub const NC_EEMPTY: c_int = -139;
#[cfg(all(feature = "4.8.0", not(feature = "4.8.1")))]
pub const NC_EFOUND: c_int = -140;
#[cfg(feature = "4.8.1")]
pub const NC_EOBJECT: c_int = -140;
#[cfg(feature = "4.8.1")]
pub const NC_ENOOBJECT: c_int = -141;
#[cfg(feature = "4.8.1")]
pub const NC_EPLUGIN: c_int = -142;

#[rustfmt::skip]
pub const NC4_LAST_ERROR: c_int = {
    #[cfg(not(feature = "4.6.0"))]
    { -131 }
    #[cfg(not(feature = "4.6.2"))]
    { -135 }
    #[cfg(all(feature = "4.6.2", not(feature = "4.7.4")))]
    { -136 }
    #[cfg(all(feature = "4.7.4", not(feature = "4.8.0")))]
    { -137 }
    #[cfg(all(feature = "4.8.0", not(feature = "4.8.1")))]
    { -140 }
    #[cfg(feature = "4.8.1")]
    { -142 }
};

pub const NC_EURL: c_int = NC_EDAPURL;
pub const NC_ECONSTRAINT: c_int = NC_EDAPCONSTRAINT;

pub const DIM_WITHOUT_VARIABLE: &[u8] = b"This is a netCDF dimension but not a netCDF variable.\0";

mod netcdf_2 {
    use super::*;

    pub const FILL_BYTE: i8 = NC_FILL_BYTE;
    pub const FILL_CHAR: u8 = NC_FILL_CHAR;
    pub const FILL_SHORT: i16 = NC_FILL_SHORT;
    pub const FILL_LONG: i32 = NC_FILL_INT;
    pub const FILL_FLOAT: f32 = NC_FILL_FLOAT;
    pub const FILL_DOUBLE: f64 = NC_FILL_DOUBLE;

    #[cfg_attr(
        feature = "4.5.0",
        deprecated(note = "Not enforced"),
        allow(deprecated)
    )]
    pub const MAX_NC_DIMS: c_int = NC_MAX_DIMS;
    #[cfg_attr(
        feature = "4.5.0",
        deprecated(note = "Not enforced"),
        allow(deprecated)
    )]
    pub const MAX_NC_ATTRS: c_int = NC_MAX_ATTRS;
    #[cfg_attr(
        feature = "4.5.0",
        deprecated(note = "Not enforced"),
        allow(deprecated)
    )]
    pub const MAX_NC_VARS: c_int = NC_MAX_VARS;
    pub const MAX_NC_NAME: c_int = NC_MAX_NAME;
    pub const MAX_VAR_DIMS: c_int = NC_MAX_VAR_DIMS;

    pub const NC_ENTOOL: c_int = NC_EMAXNAME;
    pub const NC_EXDR: c_int = -32;
    pub const NC_SYSERR: c_int = -31;

    pub const NC_FATAL: c_int = 1;
    pub const NC_VERBOSE: c_int = 2;
}

pub use netcdf_2::*;

pub const NC_TURN_OFF_LOGGING: c_int = -1;
