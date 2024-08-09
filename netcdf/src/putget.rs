use super::types::*;
use crate::{error::Result, utils::checked_with_lock};

fn conversion_supported(from: &NcVariableType, to: &NcVariableType) -> bool {
    match (from, to) {
        (
            &NcVariableType::Int(_) | &NcVariableType::Float(_),
            &NcVariableType::Int(_) | &NcVariableType::Float(_),
        ) => true,
        (from, to) => from == to,
    }
}

#[allow(clippy::too_many_lines)]
fn get_vars_mono(
    var: &crate::Variable,
    tp: &NcVariableType,
    start: &[usize],
    count: &[usize],
    stride: &[isize],
    values: *mut std::ffi::c_void,
) -> Result<()> {
    let var_tp = var.vartype();

    if !conversion_supported(&var_tp, tp) {
        return Err("Conversion not supported".into());
    }

    match tp {
        NcVariableType::Int(IntType::U8) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_get_vars_uchar(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Int(IntType::I8) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_get_vars_schar(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Int(IntType::U16) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_get_vars_ushort(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Int(IntType::I16) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_get_vars_short(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Int(IntType::U32) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_get_vars_uint(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Int(IntType::I32) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_get_vars_int(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Int(IntType::U64) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_get_vars_ulonglong(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Int(IntType::I64) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_get_vars_longlong(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Float(FloatType::F32) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_get_vars_float(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Float(FloatType::F64) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_get_vars_double(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::String => checked_with_lock(|| unsafe {
            netcdf_sys::nc_get_vars_string(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Char
        | NcVariableType::Opaque(_)
        | NcVariableType::Compound(_)
        | NcVariableType::Vlen(_) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_get_vars(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Enum(_) => {
            // TODO: Safety hole if reading a file where enum values are
            // invalid (e.g. uninitialised, set by nc_put_vars using invalid args)
            checked_with_lock(|| unsafe {
                netcdf_sys::nc_get_vars(
                    var.ncid,
                    var.varid,
                    start.as_ptr(),
                    count.as_ptr(),
                    stride.as_ptr(),
                    values.cast(),
                )
            })
        }
    }
}

pub(crate) fn get_vars<T>(
    var: &crate::Variable,
    tp: &NcVariableType,
    start: &[usize],
    count: &[usize],
    stride: &[isize],
    values: *mut T,
) -> crate::error::Result<()> {
    assert_eq!(
        tp.size(),
        std::mem::size_of::<T>(),
        "Size mismatch between type descriptor and type pointer"
    );

    get_vars_mono(var, tp, start, count, stride, values.cast())
}

/// Non-typechecked version of get_vars
/// to support getting a bag of bytes
pub fn get_raw_values_into(
    variable: &crate::Variable,
    buffer: &mut [u8],
    extents: crate::Extents,
) -> crate::error::Result<()> {
    let dims = variable.dimensions();
    let (start, count, stride) = extents.get_start_count_stride(dims)?;
    let number_of_elements = count.iter().copied().fold(1_usize, usize::saturating_mul);
    let varsize = variable.vartype().size();
    if number_of_elements * varsize != buffer.len() {
        return Err("Buffer is not of requisite size".into());
    }
    checked_with_lock(|| unsafe {
        netcdf_sys::nc_get_vars(
            variable.ncid,
            variable.varid,
            start.as_ptr(),
            count.as_ptr(),
            stride.as_ptr(),
            buffer.as_mut_ptr().cast(),
        )
    })?;

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn put_vars_mono(
    var: &mut crate::VariableMut,
    tp: &NcVariableType,
    start: &[usize],
    count: &[usize],
    stride: &[isize],
    values: *const std::ffi::c_char,
) -> crate::error::Result<()> {
    let var_tp = var.vartype();

    if !conversion_supported(tp, &var_tp) {
        return Err("Conversion not supported".into());
    }

    match tp {
        NcVariableType::Int(IntType::U8) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_put_vars_uchar(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Int(IntType::I8) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_put_vars_schar(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Int(IntType::U16) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_put_vars_ushort(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Int(IntType::I16) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_put_vars_short(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Int(IntType::U32) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_put_vars_uint(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Int(IntType::I32) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_put_vars_int(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Int(IntType::U64) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_put_vars_ulonglong(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Int(IntType::I64) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_put_vars_longlong(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Float(FloatType::F32) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_put_vars_float(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::Float(FloatType::F64) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_put_vars_double(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
        NcVariableType::String => {
            assert_eq!(
                values.align_offset(std::mem::align_of::<*const std::ffi::c_char>()),
                0,
                "Pointer is not aligned"
            );
            #[allow(clippy::cast_ptr_alignment)]
            let ptr = values.cast::<*const std::ffi::c_char>().cast_mut();

            checked_with_lock(|| unsafe {
                netcdf_sys::nc_put_vars_string(
                    var.ncid,
                    var.varid,
                    start.as_ptr(),
                    count.as_ptr(),
                    stride.as_ptr(),
                    ptr,
                )
            })
        }
        NcVariableType::Char
        | NcVariableType::Opaque(_)
        | NcVariableType::Compound(_)
        | NcVariableType::Enum(_)
        | NcVariableType::Vlen(_) => checked_with_lock(|| unsafe {
            netcdf_sys::nc_put_vars(
                var.ncid,
                var.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                values.cast(),
            )
        }),
    }
}

pub(crate) fn put_vars<T>(
    var: &mut crate::VariableMut,
    tp: &NcVariableType,
    start: &[usize],
    count: &[usize],
    stride: &[isize],
    values: *const T,
) -> crate::error::Result<()> {
    assert_eq!(
        tp.size(),
        std::mem::size_of::<T>(),
        "Size mismatch between type descriptor and type pointer"
    );
    put_vars_mono(var, tp, start, count, stride, values.cast())
}
