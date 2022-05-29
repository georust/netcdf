use std::os::raw::{c_char, c_int, c_longlong, c_uint, c_void};

use super::nc_type;

pub const NC_DISPATCH_VERSION: usize = 5;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct NC_Dispatch {
    pub model: c_int,
    pub dispatch_version: c_int,

    pub create: Option<
        unsafe extern "C" fn(
            path: *const c_char,
            cmode: c_int,
            initialsz: usize,
            basepe: c_int,
            chunksizehintp: *mut usize,
            parameters: *mut c_void,
            table: *const NC_Dispatch,
            ncid: c_int,
        ) -> c_int,
    >,
    pub open: Option<
        unsafe extern "C" fn(
            path: *const c_char,
            mode: c_int,
            basepe: c_int,
            chunksizehintp: *mut usize,
            parameters: *mut c_void,
            table: *const NC_Dispatch,
            ncid: c_int,
        ) -> c_int,
    >,
    pub redef: Option<unsafe extern "C" fn(c_int) -> c_int>,
    pub _enddef: Option<unsafe extern "C" fn(c_int, usize, usize, usize, usize) -> c_int>,
    pub sync: Option<unsafe extern "C" fn(c_int) -> c_int>,
    pub abort: Option<unsafe extern "C" fn(c_int) -> c_int>,
    pub close: Option<unsafe extern "C" fn(c_int, *mut c_void) -> c_int>,
    pub set_fill: Option<unsafe extern "C" fn(c_int, c_int, *mut c_int) -> c_int>,
    pub inq_format: Option<unsafe extern "C" fn(c_int, *mut c_int) -> c_int>,
    pub inq_format_extended: Option<unsafe extern "C" fn(c_int, *mut c_int, *mut c_int) -> c_int>,
    pub inq: Option<
        unsafe extern "C" fn(c_int, *mut c_int, *mut c_int, *mut c_int, *mut c_int) -> c_int,
    >,
    pub inq_type: Option<unsafe extern "C" fn(c_int, nc_type, *mut c_char, *mut usize) -> c_int>,
    pub def_dim: Option<unsafe extern "C" fn(c_int, *const c_char, usize, *mut c_int) -> c_int>,
    pub inq_dimid: Option<unsafe extern "C" fn(c_int, *const c_char, *mut c_int) -> c_int>,
    pub inq_dim: Option<unsafe extern "C" fn(c_int, c_int, *mut c_char, *mut usize) -> c_int>,
    pub inq_unlimdim: Option<unsafe extern "C" fn(ncid: c_int, unlimdimidp: *mut c_int) -> c_int>,
    pub rename_dim: Option<unsafe extern "C" fn(c_int, c_int, *const c_char) -> c_int>,
    pub inq_att: Option<
        unsafe extern "C" fn(c_int, c_int, *const c_char, *mut nc_type, *mut usize) -> c_int,
    >,
    pub inq_attid: Option<unsafe extern "C" fn(c_int, c_int, *const c_char, *mut c_int) -> c_int>,
    pub inq_attname: Option<unsafe extern "C" fn(c_int, c_int, c_int, *mut c_char) -> c_int>,
    pub rename_att:
        Option<unsafe extern "C" fn(c_int, c_int, *const c_char, *const c_char) -> c_int>,
    pub del_att: Option<unsafe extern "C" fn(c_int, c_int, *const c_char) -> c_int>,
    pub get_att:
        Option<unsafe extern "C" fn(c_int, c_int, *const c_char, *mut c_void, nc_type) -> c_int>,
    pub put_att: Option<
        unsafe extern "C" fn(
            c_int,
            c_int,
            *const c_char,
            nc_type,
            usize,
            *const c_void,
            nc_type,
        ) -> c_int,
    >,
    pub def_var: Option<
        unsafe extern "C" fn(
            c_int,
            *const c_char,
            nc_type,
            c_int,
            *const c_int,
            *mut c_int,
        ) -> c_int,
    >,
    pub inq_varid: Option<unsafe extern "C" fn(c_int, *const c_char, *mut c_int) -> c_int>,
    pub rename_var: Option<unsafe extern "C" fn(c_int, c_int, *const c_char) -> c_int>,
    pub get_vara: Option<
        unsafe extern "C" fn(
            c_int,
            c_int,
            *const usize,
            *const usize,
            *mut c_void,
            nc_type,
        ) -> c_int,
    >,
    pub put_vara: Option<
        unsafe extern "C" fn(
            c_int,
            c_int,
            *const usize,
            *const usize,
            *const c_void,
            nc_type,
        ) -> c_int,
    >,
    pub get_vars: Option<
        unsafe extern "C" fn(
            c_int,
            c_int,
            *const usize,
            *const usize,
            *const isize,
            *mut c_void,
            nc_type,
        ) -> c_int,
    >,
    pub put_vars: Option<
        unsafe extern "C" fn(
            c_int,
            c_int,
            *const usize,
            *const usize,
            *const isize,
            *const c_void,
            nc_type,
        ) -> c_int,
    >,
    pub get_varm: Option<
        unsafe extern "C" fn(
            c_int,
            c_int,
            *const usize,
            *const usize,
            *const isize,
            *const isize,
            *mut c_void,
            nc_type,
        ) -> c_int,
    >,
    pub put_varm: Option<
        unsafe extern "C" fn(
            c_int,
            c_int,
            *const usize,
            *const usize,
            *const isize,
            *const isize,
            *const c_void,
            nc_type,
        ) -> c_int,
    >,
    pub inq_var_all: Option<
        unsafe extern "C" fn(
            ncid: c_int,
            varid: c_int,
            name: *mut c_char,
            xtypep: *mut nc_type,
            ndimsp: *mut c_int,
            dimidsp: *mut c_int,
            nattsp: *mut c_int,
            shufflep: *mut c_int,
            deflatep: *mut c_int,
            deflate_levelp: *mut c_int,
            fletcher32p: *mut c_int,
            contiguousp: *mut c_int,
            chunksizesp: *mut usize,
            no_fill: *mut c_int,
            fill_valuep: *mut c_void,
            endiannessp: *mut c_int,
            idp: *mut c_uint,
            nparamsp: *mut usize,
            params: *mut c_uint,
        ) -> c_int,
    >,
    pub var_par_access: Option<unsafe extern "C" fn(c_int, c_int, c_int) -> c_int>,
    pub def_var_fill: Option<unsafe extern "C" fn(c_int, c_int, c_int, *const c_void) -> c_int>,
    pub show_metadata: Option<unsafe extern "C" fn(c_int) -> c_int>,
    pub inq_unlimdims: Option<unsafe extern "C" fn(c_int, *mut c_int, *mut c_int) -> c_int>,
    pub inq_ncid: Option<unsafe extern "C" fn(c_int, *const c_char, *mut c_int) -> c_int>,
    pub inq_grps: Option<unsafe extern "C" fn(c_int, *mut c_int, *mut c_int) -> c_int>,
    pub inq_grpname: Option<unsafe extern "C" fn(c_int, *mut c_char) -> c_int>,
    pub inq_grpname_full: Option<unsafe extern "C" fn(c_int, *mut usize, *mut c_char) -> c_int>,
    pub inq_grp_parent: Option<unsafe extern "C" fn(c_int, *mut c_int) -> c_int>,
    pub inq_grp_full_ncid: Option<unsafe extern "C" fn(c_int, *const c_char, *mut c_int) -> c_int>,
    pub inq_varids:
        Option<unsafe extern "C" fn(_: c_int, nvars: *mut c_int, _: *mut c_int) -> c_int>,
    pub inq_dimids:
        Option<unsafe extern "C" fn(_: c_int, ndims: *mut c_int, _: *mut c_int, _: c_int) -> c_int>,
    pub inq_typeids:
        Option<unsafe extern "C" fn(_: c_int, ntypes: *mut c_int, _: *mut c_int) -> c_int>,
    pub inq_type_equal:
        Option<unsafe extern "C" fn(c_int, nc_type, c_int, nc_type, *mut c_int) -> c_int>,
    pub def_grp: Option<unsafe extern "C" fn(c_int, *const c_char, *mut c_int) -> c_int>,
    pub rename_grp: Option<unsafe extern "C" fn(c_int, *const c_char) -> c_int>,
    pub inq_user_type: Option<
        unsafe extern "C" fn(
            c_int,
            nc_type,
            *mut c_char,
            *mut usize,
            *mut nc_type,
            *mut usize,
            *mut c_int,
        ) -> c_int,
    >,
    pub inq_typeid: Option<unsafe extern "C" fn(c_int, *const c_char, *mut nc_type) -> c_int>,
    pub def_compound:
        Option<unsafe extern "C" fn(c_int, usize, *const c_char, *mut nc_type) -> c_int>,
    pub insert_compound:
        Option<unsafe extern "C" fn(c_int, nc_type, *const c_char, usize, nc_type) -> c_int>,
    pub insert_array_compound: Option<
        unsafe extern "C" fn(
            c_int,
            nc_type,
            *const c_char,
            usize,
            nc_type,
            c_int,
            *const c_int,
        ) -> c_int,
    >,
    pub inq_compound_field: Option<
        unsafe extern "C" fn(
            c_int,
            nc_type,
            c_int,
            *mut c_char,
            *mut usize,
            *mut nc_type,
            *mut c_int,
            *mut c_int,
        ) -> c_int,
    >,
    pub inq_compound_fieldindex:
        Option<unsafe extern "C" fn(c_int, nc_type, *const c_char, *mut c_int) -> c_int>,
    pub def_vlen: Option<
        unsafe extern "C" fn(
            _: c_int,
            _: *const c_char,
            base_typeid: nc_type,
            _: *mut nc_type,
        ) -> c_int,
    >,
    pub put_vlen_element:
        Option<unsafe extern "C" fn(c_int, c_int, *mut c_void, usize, *const c_void) -> c_int>,
    pub get_vlen_element:
        Option<unsafe extern "C" fn(c_int, c_int, *const c_void, *mut usize, *mut c_void) -> c_int>,
    pub def_enum:
        Option<unsafe extern "C" fn(c_int, nc_type, *const c_char, *mut nc_type) -> c_int>,
    pub insert_enum:
        Option<unsafe extern "C" fn(c_int, nc_type, *const c_char, *const c_void) -> c_int>,
    pub inq_enum_member:
        Option<unsafe extern "C" fn(c_int, nc_type, c_int, *mut c_char, *mut c_void) -> c_int>,
    pub inq_enum_ident:
        Option<unsafe extern "C" fn(c_int, nc_type, c_longlong, *mut c_char) -> c_int>,
    pub def_opaque:
        Option<unsafe extern "C" fn(c_int, usize, *const c_char, *mut nc_type) -> c_int>,
    pub def_var_deflate: Option<unsafe extern "C" fn(c_int, c_int, c_int, c_int, c_int) -> c_int>,
    pub def_var_fletcher32: Option<unsafe extern "C" fn(c_int, c_int, c_int) -> c_int>,
    pub def_var_chunking: Option<unsafe extern "C" fn(c_int, c_int, c_int, *const usize) -> c_int>,
    pub def_var_endian: Option<unsafe extern "C" fn(c_int, c_int, c_int) -> c_int>,
    pub def_var_filter:
        Option<unsafe extern "C" fn(c_int, c_int, c_uint, usize, *const c_uint) -> c_int>,
    pub set_var_chunk_cache: Option<unsafe extern "C" fn(c_int, c_int, usize, usize, f32) -> c_int>,
    pub get_var_chunk_cache: Option<
        unsafe extern "C" fn(
            ncid: c_int,
            varid: c_int,
            sizep: *mut usize,
            nelemsp: *mut usize,
            preemptionp: *mut f32,
        ) -> c_int,
    >,
    pub inq_var_filter_ids: Option<
        unsafe extern "C" fn(
            ncid: c_int,
            varid: c_int,
            nfilters: *mut usize,
            filterids: *mut c_uint,
        ) -> c_int,
    >,
    pub inq_var_filter_info: Option<
        unsafe extern "C" fn(
            ncid: c_int,
            varid: c_int,
            id: c_uint,
            nparams: *mut usize,
            params: *mut c_uint,
        ) -> c_int,
    >,
    pub def_var_quantize: Option<
        unsafe extern "C" fn(ncid: c_int, varid: c_int, quantize_mode: c_int, nsd: c_int) -> c_int,
    >,
    pub inq_var_quantize: Option<
        unsafe extern "C" fn(
            ncid: c_int,
            varid: c_int,
            quantize_modep: *mut c_int,
            nsdp: *mut c_int,
        ) -> c_int,
    >,
    pub inq_filter_avail: Option<unsafe extern "C" fn(ncid: c_int, id: c_uint) -> c_int>,
}

extern "C" {
    pub fn NC_RO_create(
        path: *const c_char,
        cmode: c_int,
        initialsz: usize,
        basepe: c_int,
        chunksizehintp: *mut usize,
        parameters: *mut c_void,
        _: *const NC_Dispatch,
        _: c_int,
    ) -> c_int;
    pub fn NC_RO_redef(ncid: c_int) -> c_int;
    pub fn NC_RO__enddef(
        ncid: c_int,
        h_minfree: usize,
        v_align: usize,
        v_minfree: usize,
        r_align: usize,
    ) -> c_int;
    pub fn NC_RO_sync(ncid: c_int) -> c_int;
    pub fn NC_RO_def_var_fill(_: c_int, _: c_int, _: c_int, _: *const c_void) -> c_int;
    pub fn NC_RO_rename_att(
        ncid: c_int,
        varid: c_int,
        name: *const c_char,
        newname: *const c_char,
    ) -> c_int;
    pub fn NC_RO_del_att(ncid: c_int, varid: c_int, _: *const c_char) -> c_int;
    pub fn NC_RO_put_att(
        ncid: c_int,
        varid: c_int,
        name: *const c_char,
        datatype: nc_type,
        len: usize,
        value: *const c_void,
        _: nc_type,
    ) -> c_int;
    pub fn NC_RO_def_var(
        ncid: c_int,
        name: *const c_char,
        xtype: nc_type,
        ndims: c_int,
        dimidsp: *const c_int,
        varidp: *mut c_int,
    ) -> c_int;
    pub fn NC_RO_rename_var(ncid: c_int, varid: c_int, name: *const c_char) -> c_int;
    pub fn NC_RO_put_vara(
        ncid: c_int,
        varid: c_int,
        start: *const usize,
        count: *const usize,
        value: *const c_void,
        _: nc_type,
    ) -> c_int;
    pub fn NC_RO_def_dim(ncid: c_int, name: *const c_char, len: usize, idp: *mut c_int) -> c_int;
    pub fn NC_RO_rename_dim(ncid: c_int, dimid: c_int, name: *const c_char) -> c_int;
    pub fn NC_RO_set_fill(ncid: c_int, fillmode: c_int, old_modep: *mut c_int) -> c_int;
    pub fn NC_NOTNC4_def_var_filter(
        _: c_int,
        _: c_int,
        _: c_uint,
        _: usize,
        _: *const c_uint,
    ) -> c_int;
    pub fn NC_NOTNC4_inq_var_filter_ids(
        ncid: c_int,
        varid: c_int,
        nfilters: *mut usize,
        filterids: *mut c_uint,
    ) -> c_int;
    pub fn NC_NOTNC4_inq_var_filter_info(
        ncid: c_int,
        varid: c_int,
        id: c_uint,
        nparams: *mut usize,
        params: *mut c_uint,
    ) -> c_int;
    pub fn NC_NOOP_inq_var_filter_ids(
        ncid: c_int,
        varid: c_int,
        nfilters: *mut usize,
        filterids: *mut c_uint,
    ) -> c_int;
    pub fn NC_NOOP_inq_var_filter_info(
        ncid: c_int,
        varid: c_int,
        id: c_uint,
        nparams: *mut usize,
        params: *mut c_uint,
    ) -> c_int;
    pub fn NC_NOOP_inq_filter_avail(ncid: c_int, id: c_uint) -> c_int;
    pub fn NC_NOTNC4_def_grp(_: c_int, _: *const c_char, _: *mut c_int) -> c_int;
    pub fn NC_NOTNC4_rename_grp(_: c_int, _: *const c_char) -> c_int;
    pub fn NC_NOTNC4_def_compound(_: c_int, _: usize, _: *const c_char, _: *mut nc_type) -> c_int;
    pub fn NC_NOTNC4_insert_compound(
        _: c_int,
        _: nc_type,
        _: *const c_char,
        _: usize,
        _: nc_type,
    ) -> c_int;
    pub fn NC_NOTNC4_insert_array_compound(
        _: c_int,
        _: nc_type,
        _: *const c_char,
        _: usize,
        _: nc_type,
        _: c_int,
        _: *const c_int,
    ) -> c_int;
    pub fn NC_NOTNC4_inq_typeid(_: c_int, _: *const c_char, _: *mut nc_type) -> c_int;
    pub fn NC_NOTNC4_inq_compound_field(
        _: c_int,
        _: nc_type,
        _: c_int,
        _: *mut c_char,
        _: *mut usize,
        _: *mut nc_type,
        _: *mut c_int,
        _: *mut c_int,
    ) -> c_int;
    pub fn NC_NOTNC4_inq_compound_fieldindex(
        _: c_int,
        _: nc_type,
        _: *const c_char,
        _: *mut c_int,
    ) -> c_int;
    pub fn NC_NOTNC4_def_vlen(
        _: c_int,
        _: *const c_char,
        base_typeid: nc_type,
        _: *mut nc_type,
    ) -> c_int;
    pub fn NC_NOTNC4_put_vlen_element(
        _: c_int,
        _: c_int,
        _: *mut c_void,
        _: usize,
        _: *const c_void,
    ) -> c_int;
    pub fn NC_NOTNC4_get_vlen_element(
        _: c_int,
        _: c_int,
        _: *const c_void,
        _: *mut usize,
        _: *mut c_void,
    ) -> c_int;
    pub fn NC_NOTNC4_def_enum(_: c_int, _: nc_type, _: *const c_char, _: *mut nc_type) -> c_int;
    pub fn NC_NOTNC4_insert_enum(_: c_int, _: nc_type, _: *const c_char, _: *const c_void)
        -> c_int;
    pub fn NC_NOTNC4_inq_enum_member(
        _: c_int,
        _: nc_type,
        _: c_int,
        _: *mut c_char,
        _: *mut c_void,
    ) -> c_int;
    pub fn NC_NOTNC4_inq_enum_ident(_: c_int, _: nc_type, _: c_longlong, _: *mut c_char) -> c_int;
    pub fn NC_NOTNC4_def_opaque(_: c_int, _: usize, _: *const c_char, _: *mut nc_type) -> c_int;
    pub fn NC_NOTNC4_def_var_deflate(_: c_int, _: c_int, _: c_int, _: c_int, _: c_int) -> c_int;
    pub fn NC_NOTNC4_def_var_fletcher32(_: c_int, _: c_int, _: c_int) -> c_int;
    pub fn NC_NOTNC4_def_var_chunking(_: c_int, _: c_int, _: c_int, _: *const usize) -> c_int;
    pub fn NC_NOTNC4_def_var_endian(_: c_int, _: c_int, _: c_int) -> c_int;
    pub fn NC_NOTNC4_set_var_chunk_cache(_: c_int, _: c_int, _: usize, _: usize, _: f32) -> c_int;
    pub fn NC_NOTNC4_get_var_chunk_cache(
        _: c_int,
        _: c_int,
        _: *mut usize,
        _: *mut usize,
        _: *mut f32,
    ) -> c_int;
    pub fn NC_NOTNC4_var_par_access(_: c_int, _: c_int, _: c_int) -> c_int;
    pub fn NC_NOTNC4_inq_ncid(_: c_int, _: *const c_char, _: *mut c_int) -> c_int;
    pub fn NC_NOTNC4_inq_grps(_: c_int, _: *mut c_int, _: *mut c_int) -> c_int;
    pub fn NC_NOTNC4_inq_grpname(_: c_int, _: *mut c_char) -> c_int;
    pub fn NC_NOTNC4_inq_grpname_full(_: c_int, _: *mut usize, _: *mut c_char) -> c_int;
    pub fn NC_NOTNC4_inq_grp_parent(_: c_int, _: *mut c_int) -> c_int;
    pub fn NC_NOTNC4_inq_grp_full_ncid(_: c_int, _: *const c_char, _: *mut c_int) -> c_int;
    pub fn NC_NOTNC4_inq_varids(_: c_int, _: *mut c_int, _: *mut c_int) -> c_int;
    pub fn NC_NOTNC4_inq_dimids(_: c_int, _: *mut c_int, _: *mut c_int, _: c_int) -> c_int;
    pub fn NC_NOTNC4_inq_typeids(_: c_int, _: *mut c_int, _: *mut c_int) -> c_int;
    pub fn NC_NOTNC4_inq_user_type(
        _: c_int,
        _: nc_type,
        _: *mut c_char,
        _: *mut usize,
        _: *mut nc_type,
        _: *mut usize,
        _: *mut c_int,
    ) -> c_int;
    pub fn NC_NOTNC4_def_var_quantize(_: c_int, _: c_int, _: c_int, _: c_int) -> c_int;
    pub fn NC_NOTNC4_inq_var_quantize(_: c_int, _: c_int, _: *mut c_int, _: *mut c_int) -> c_int;
    pub fn NC_NOTNC3_get_varm(
        ncid: c_int,
        varid: c_int,
        start: *const usize,
        edges: *const usize,
        stride: *const isize,
        imapp: *const isize,
        value0: *mut c_void,
        memtype: nc_type,
    ) -> c_int;
    pub fn NC_NOTNC3_put_varm(
        ncid: c_int,
        varid: c_int,
        start: *const usize,
        edges: *const usize,
        stride: *const isize,
        imapp: *const isize,
        value0: *const c_void,
        memtype: nc_type,
    ) -> c_int;
}
