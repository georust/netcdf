//! Variables in the netcdf file
#![allow(clippy::similar_names)]
use std::convert::TryInto;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::marker::Sized;
use std::mem::MaybeUninit;
use std::ptr::addr_of;

use super::attribute::AttrValue;
use super::attribute::Attribute;
use super::dimension::Dimension;
use super::error;
use super::extent::Extents;
use super::types::VariableType;
#[cfg(feature = "ndarray")]
use ndarray::ArrayD;
use netcdf_sys::*;

#[allow(clippy::doc_markdown)]
/// This struct defines a `netCDF` variable.
#[derive(Debug, Clone)]
pub struct Variable<'g> {
    /// The variable name
    pub(crate) dimensions: Vec<Dimension<'g>>,
    /// the `netCDF` variable type identifier (from netcdf-sys)
    pub(crate) vartype: nc_type,
    pub(crate) ncid: nc_type,
    pub(crate) varid: nc_type,
    pub(crate) _group: PhantomData<&'g nc_type>,
}

#[derive(Debug)]
/// Mutable access to a variable.
#[allow(clippy::module_name_repetitions)]
pub struct VariableMut<'g>(
    pub(crate) Variable<'g>,
    pub(crate) PhantomData<&'g mut nc_type>,
);

impl<'g> std::ops::Deref for VariableMut<'g> {
    type Target = Variable<'g>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Enum for variables endianness
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Endianness {
    /// Native endianness, depends on machine architecture (x86_64 is Little)
    Native,
    /// Lille endian
    Little,
    /// Big endian
    Big,
}

#[allow(clippy::len_without_is_empty)]
impl<'g> Variable<'g> {
    pub(crate) fn find_from_name(ncid: nc_type, name: &str) -> error::Result<Option<Variable<'g>>> {
        let cname = super::utils::short_name_to_bytes(name)?;
        let mut varid = 0;
        let e =
            unsafe { super::with_lock(|| nc_inq_varid(ncid, cname.as_ptr().cast(), &mut varid)) };
        if e == NC_ENOTVAR {
            return Ok(None);
        }
        error::checked(e)?;

        let mut xtype = 0;
        let mut ndims = 0;
        unsafe {
            error::checked(super::with_lock(|| {
                nc_inq_var(
                    ncid,
                    varid,
                    std::ptr::null_mut(),
                    &mut xtype,
                    &mut ndims,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            }))?;
        }
        let mut dimids = vec![0; ndims.try_into()?];
        unsafe {
            error::checked(super::with_lock(|| {
                nc_inq_vardimid(ncid, varid, dimids.as_mut_ptr())
            }))?;
        }
        let dimensions = super::dimension::dimensions_from_variable(ncid, varid)?
            .collect::<error::Result<Vec<_>>>()?;

        Ok(Some(Variable {
            dimensions,
            ncid,
            varid,
            vartype: xtype,
            _group: PhantomData,
        }))
    }

    /// Get name of variable
    pub fn name(&self) -> String {
        let mut name = vec![0_u8; NC_MAX_NAME as usize + 1];
        unsafe {
            error::checked(super::with_lock(|| {
                nc_inq_varname(self.ncid, self.varid, name.as_mut_ptr().cast())
            }))
            .unwrap();
        }
        let zeropos = name.iter().position(|&x| x == 0).unwrap_or(name.len());
        name.resize(zeropos, 0);

        String::from_utf8(name).expect("Variable name contained invalid sequence")
    }
    /// Get an attribute of this variable
    pub fn attribute<'a>(&'a self, name: &str) -> Option<Attribute<'a>> {
        // Need to lock when reading the first attribute (per variable)
        Attribute::find_from_name(self.ncid, Some(self.varid), name)
            .expect("Could not retrieve attribute")
    }
    /// Iterator over all the attributes of this variable
    pub fn attributes(&self) -> impl Iterator<Item = Attribute> {
        // Need to lock when reading the first attribute (per variable)
        crate::attribute::AttributeIterator::new(self.ncid, Some(self.varid))
            .expect("Could not get attributes")
            .map(Result::unwrap)
    }
    /// Dimensions for a variable
    pub fn dimensions(&self) -> &[Dimension] {
        &self.dimensions
    }
    /// Get the type of this variable
    pub fn vartype(&self) -> VariableType {
        VariableType::from_id(self.ncid, self.vartype).unwrap()
    }
    /// Get current length of the variable
    pub fn len(&self) -> usize {
        self.dimensions
            .iter()
            .map(Dimension::len)
            .fold(1_usize, usize::saturating_mul)
    }
    /// Get endianness of the variable.
    ///
    /// # Errors
    ///
    /// Not a `netCDF-4` file
    pub fn endian_value(&self) -> error::Result<Endianness> {
        let mut e: nc_type = 0;
        unsafe {
            error::checked(super::with_lock(|| {
                nc_inq_var_endian(self.ncid, self.varid, &mut e)
            }))?;
        }
        match e {
            NC_ENDIAN_NATIVE => Ok(Endianness::Native),
            NC_ENDIAN_LITTLE => Ok(Endianness::Little),
            NC_ENDIAN_BIG => Ok(Endianness::Big),
            _ => Err(NC_EVARMETA.into()),
        }
    }
}
impl<'g> VariableMut<'g> {
    /// Sets compression on the variable. Must be set before filling in data.
    ///
    /// `deflate_level` can take a value 0..=9, with 0 being no
    /// compression (good for CPU bound tasks), and 9 providing the
    /// highest compression level (good for memory bound tasks)
    ///
    /// # Errors
    ///
    /// Not a `netcdf-4` file or `deflate_level` not valid
    pub fn compression(&mut self, deflate_level: nc_type) -> error::Result<()> {
        unsafe {
            error::checked(super::with_lock(|| {
                nc_def_var_deflate(
                    self.ncid,
                    self.varid,
                    <_>::from(false),
                    <_>::from(true),
                    deflate_level,
                )
            }))?;
        }

        Ok(())
    }

    /// Set chunking for variable. Must be set before inserting data
    ///
    /// Use this when reading or writing smaller units of the hypercube than
    /// the full dimensions lengths, to change how the variable is stored in
    /// the file. This has no effect on the memory order when reading/putting
    /// a buffer.
    ///
    /// # Errors
    ///
    /// Not a `netCDF-4` file or invalid chunksize
    pub fn chunking(&mut self, chunksize: &[usize]) -> error::Result<()> {
        if self.dimensions.is_empty() {
            // Can't really set chunking, would lead to segfault
            return Ok(());
        }
        if chunksize.len() != self.dimensions.len() {
            return Err(error::Error::SliceLen);
        }
        let len = chunksize
            .iter()
            .copied()
            .fold(1_usize, usize::saturating_mul);
        if len == usize::max_value() {
            return Err(error::Error::Overflow);
        }
        unsafe {
            error::checked(super::with_lock(|| {
                nc_def_var_chunking(self.ncid, self.varid, NC_CHUNKED, chunksize.as_ptr())
            }))?;
        }

        Ok(())
    }
}

mod sealed {
    pub trait Sealed {}
}

#[allow(clippy::doc_markdown)]
/// This trait allow an implicit cast when fetching
/// a netCDF variable. These methods are not be called
/// directly, but used through methods on `Variable`
///
/// # Safety
/// This trait maps directly to netCDF semantics and needs
/// to upheld invariants therein.
/// This trait is sealed and can not be implemented for
/// types outside this crate
pub trait NcPutGet: sealed::Sealed
where
    Self: Sized,
{
    /// Constant corresponding to a netcdf type
    const NCTYPE: nc_type;

    /// Returns a single indexed value of the variable as Self
    ///
    /// # Safety
    ///
    /// Requires `indices` to be of a valid length
    unsafe fn get_var1(variable: &Variable, start: &[usize]) -> error::Result<Self>;

    #[allow(clippy::doc_markdown)]
    /// Put a single value into a netCDF variable
    ///
    /// # Safety
    ///
    /// Requires `indices` to be of a valid length
    unsafe fn put_var1(
        variable: &mut VariableMut,
        start: &[usize],
        value: Self,
    ) -> error::Result<()>;

    /// Get multiple values at once, without checking the validity of
    /// `indices` or `slice_len`
    ///
    /// # Safety
    ///
    /// Requires `values` to be of at least size `slice_len.product()`,
    /// `indices` and `slice_len` to be of a valid length
    unsafe fn get_vara(
        variable: &Variable,
        start: &[usize],
        count: &[usize],
        values: *mut Self,
    ) -> error::Result<()>;

    #[allow(clippy::doc_markdown)]
    /// put a SLICE of values into a netCDF variable at the given index
    ///
    /// # Safety
    ///
    /// Requires `indices` and `slice_len` to be of a valid length
    unsafe fn put_vara(
        variable: &mut VariableMut,
        start: &[usize],
        count: &[usize],
        values: &[Self],
    ) -> error::Result<()>;

    /// get a SLICE of values into the variable, with the source
    /// strided by `stride`
    ///
    /// # Safety
    ///
    /// `values` must contain space for all the data,
    /// `indices`, `slice_len`, and `stride` must be of
    /// at least dimension length size.
    unsafe fn get_vars(
        variable: &Variable,
        start: &[usize],
        count: &[usize],
        stride: &[isize],
        values: *mut Self,
    ) -> error::Result<()>;

    /// put a SLICE of values into the variable, with the destination
    /// strided by `stride`
    ///
    /// # Safety
    ///
    /// `values` must contain space for all the data,
    /// `indices`, `slice_len`, and `stride` must be of
    /// at least dimension length size.
    unsafe fn put_vars(
        variable: &mut VariableMut,
        start: &[usize],
        count: &[usize],
        stride: &[isize],
        values: *const Self,
    ) -> error::Result<()>;

    /// get a SLICE of values into the variable, with the source
    /// strided by `stride`, mapped by `map`
    ///
    /// # Safety
    ///
    /// `values` must contain space for all the data,
    /// `indices`, `slice_len`, and `stride` must be of
    /// at least dimension length size.
    unsafe fn get_varm(
        variable: &Variable,
        start: &[usize],
        count: &[usize],
        stride: &[isize],
        map: &[isize],
        values: *mut Self,
    ) -> error::Result<()>;

    /// put a SLICE of values into the variable, with the destination
    /// strided by `stride`
    ///
    /// # Safety
    ///
    /// `values` must contain space for all the data,
    /// `indices`, `slice_len`, and `stride` must be of
    /// at least dimension length size.
    unsafe fn put_varm(
        variable: &mut VariableMut,
        start: &[usize],
        count: &[usize],
        stride: &[isize],
        map: &[isize],
        values: *const Self,
    ) -> error::Result<()>;
}

#[allow(clippy::doc_markdown)]
/// This macro implements the trait NcPutGet for the type `sized_type`.
///
/// The use of this macro reduce code duplication for the implementation of NcPutGet
/// for the common numeric types (i32, f32 ...): they only differs by the name of the
/// C function used to fetch values from the NetCDF variable (eg: `nc_get_var_ushort`, ...).
macro_rules! impl_numeric {
    (
        $sized_type: ty,
        $nc_type: ident,
        $nc_get_var: ident,
        $nc_get_vara_type: ident,
        $nc_get_var1_type: ident,
        $nc_put_var1_type: ident,
        $nc_put_vara_type: ident,
        $nc_get_vars_type: ident,
        $nc_put_vars_type: ident,
        $nc_get_varm_type: ident,
        $nc_put_varm_type: ident,
    ) => {
        impl sealed::Sealed for $sized_type {}
        #[allow(clippy::use_self)] // False positives
        impl NcPutGet for $sized_type {
            const NCTYPE: nc_type = $nc_type;

            // fetch ONE value from variable using `$nc_get_var1`
            unsafe fn get_var1(variable: &Variable, start: &[usize]) -> error::Result<Self> {
                let mut buff: MaybeUninit<Self> = MaybeUninit::uninit();
                error::checked(super::with_lock(|| {
                    $nc_get_var1_type(
                        variable.ncid,
                        variable.varid,
                        start.as_ptr(),
                        buff.as_mut_ptr(),
                    )
                }))?;
                Ok(buff.assume_init())
            }

            // put a SINGLE value into a netCDF variable at the given index
            unsafe fn put_var1(
                variable: &mut VariableMut,
                start: &[usize],
                value: Self,
            ) -> error::Result<()> {
                error::checked(super::with_lock(|| {
                    $nc_put_var1_type(
                        variable.ncid,
                        variable.varid,
                        start.as_ptr(),
                        addr_of!(value),
                    )
                }))
            }

            unsafe fn get_vara(
                variable: &Variable,
                start: &[usize],
                count: &[usize],
                values: *mut Self,
            ) -> error::Result<()> {
                error::checked(super::with_lock(|| {
                    $nc_get_vara_type(
                        variable.ncid,
                        variable.varid,
                        start.as_ptr(),
                        count.as_ptr(),
                        values,
                    )
                }))
            }

            // put a SLICE of values into a netCDF variable at the given index
            unsafe fn put_vara(
                variable: &mut VariableMut,
                start: &[usize],
                count: &[usize],
                values: &[Self],
            ) -> error::Result<()> {
                error::checked(super::with_lock(|| {
                    $nc_put_vara_type(
                        variable.ncid,
                        variable.varid,
                        start.as_ptr(),
                        count.as_ptr(),
                        values.as_ptr(),
                    )
                }))
            }

            unsafe fn get_vars(
                variable: &Variable,
                start: &[usize],
                count: &[usize],
                strides: &[isize],
                values: *mut Self,
            ) -> error::Result<()> {
                error::checked(super::with_lock(|| {
                    $nc_get_vars_type(
                        variable.ncid,
                        variable.varid,
                        start.as_ptr(),
                        count.as_ptr(),
                        strides.as_ptr(),
                        values,
                    )
                }))
            }

            unsafe fn put_vars(
                variable: &mut VariableMut,
                start: &[usize],
                count: &[usize],
                stride: &[isize],
                values: *const Self,
            ) -> error::Result<()> {
                error::checked(super::with_lock(|| {
                    $nc_put_vars_type(
                        variable.ncid,
                        variable.varid,
                        start.as_ptr(),
                        count.as_ptr(),
                        stride.as_ptr(),
                        values,
                    )
                }))
            }

            unsafe fn get_varm(
                variable: &Variable,
                start: &[usize],
                count: &[usize],
                stride: &[isize],
                map: &[isize],
                values: *mut Self,
            ) -> error::Result<()> {
                error::checked(super::with_lock(|| {
                    $nc_get_varm_type(
                        variable.ncid,
                        variable.varid,
                        start.as_ptr(),
                        count.as_ptr(),
                        stride.as_ptr(),
                        map.as_ptr(),
                        values,
                    )
                }))
            }

            unsafe fn put_varm(
                variable: &mut VariableMut,
                start: &[usize],
                count: &[usize],
                stride: &[isize],
                map: &[isize],
                values: *const Self,
            ) -> error::Result<()> {
                error::checked(super::with_lock(|| {
                    $nc_put_varm_type(
                        variable.ncid,
                        variable.varid,
                        start.as_ptr(),
                        count.as_ptr(),
                        stride.as_ptr(),
                        map.as_ptr(),
                        values,
                    )
                }))
            }
        }
    };
}
impl_numeric!(
    u8,
    NC_UBYTE,
    nc_get_var_uchar,
    nc_get_vara_uchar,
    nc_get_var1_uchar,
    nc_put_var1_uchar,
    nc_put_vara_uchar,
    nc_get_vars_uchar,
    nc_put_vars_uchar,
    nc_get_varm_uchar,
    nc_put_varm_uchar,
);

impl_numeric!(
    i8,
    NC_BYTE,
    nc_get_var_schar,
    nc_get_vara_schar,
    nc_get_var1_schar,
    nc_put_var1_schar,
    nc_put_vara_schar,
    nc_get_vars_schar,
    nc_put_vars_schar,
    nc_get_varm_schar,
    nc_put_varm_schar,
);

impl_numeric!(
    i16,
    NC_SHORT,
    nc_get_var_short,
    nc_get_vara_short,
    nc_get_var1_short,
    nc_put_var1_short,
    nc_put_vara_short,
    nc_get_vars_short,
    nc_put_vars_short,
    nc_get_varm_short,
    nc_put_varm_short,
);

impl_numeric!(
    u16,
    NC_USHORT,
    nc_get_var_ushort,
    nc_get_vara_ushort,
    nc_get_var1_ushort,
    nc_put_var1_ushort,
    nc_put_vara_ushort,
    nc_get_vars_ushort,
    nc_put_vars_ushort,
    nc_get_varm_ushort,
    nc_put_varm_ushort,
);

impl_numeric!(
    i32,
    NC_INT,
    nc_get_var_int,
    nc_get_vara_int,
    nc_get_var1_int,
    nc_put_var1_int,
    nc_put_vara_int,
    nc_get_vars_int,
    nc_put_vars_int,
    nc_get_varm_int,
    nc_put_varm_int,
);

impl_numeric!(
    u32,
    NC_UINT,
    nc_get_var_uint,
    nc_get_vara_uint,
    nc_get_var1_uint,
    nc_put_var1_uint,
    nc_put_vara_uint,
    nc_get_vars_uint,
    nc_put_vars_uint,
    nc_get_varm_uint,
    nc_put_varm_uint,
);

impl_numeric!(
    i64,
    NC_INT64,
    nc_get_var_longlong,
    nc_get_vara_longlong,
    nc_get_var1_longlong,
    nc_put_var1_longlong,
    nc_put_vara_longlong,
    nc_get_vars_longlong,
    nc_put_vars_longlong,
    nc_get_varm_longlong,
    nc_put_varm_longlong,
);

impl_numeric!(
    u64,
    NC_UINT64,
    nc_get_var_ulonglong,
    nc_get_vara_ulonglong,
    nc_get_var1_ulonglong,
    nc_put_var1_ulonglong,
    nc_put_vara_ulonglong,
    nc_get_vars_ulonglong,
    nc_put_vars_ulonglong,
    nc_get_varm_ulonglong,
    nc_put_varm_ulonglong,
);

impl_numeric!(
    f32,
    NC_FLOAT,
    nc_get_var_float,
    nc_get_vara_float,
    nc_get_var1_float,
    nc_put_var1_float,
    nc_put_vara_float,
    nc_get_vars_float,
    nc_put_vars_float,
    nc_get_varm_float,
    nc_put_varm_float,
);

impl_numeric!(
    f64,
    NC_DOUBLE,
    nc_get_var_double,
    nc_get_vara_double,
    nc_get_var1_double,
    nc_put_var1_double,
    nc_put_vara_double,
    nc_get_vars_double,
    nc_put_vars_double,
    nc_get_varm_double,
    nc_put_varm_double,
);

/// Holds the contents of a netcdf string. Use deref to get a `CStr`
struct NcString {
    data: *mut std::os::raw::c_char,
}
impl NcString {
    /// Create an `NcString`
    unsafe fn from_ptr(ptr: *mut i8) -> Self {
        Self { data: ptr }
    }
}
impl Drop for NcString {
    fn drop(&mut self) {
        unsafe {
            error::checked(super::with_lock(|| nc_free_string(1, &mut self.data))).unwrap();
        }
    }
}
impl std::ops::Deref for NcString {
    type Target = CStr;
    fn deref(&self) -> &Self::Target {
        unsafe { CStr::from_ptr(self.data) }
    }
}

impl<'g> VariableMut<'g> {
    /// Adds an attribute to the variable
    pub fn add_attribute<T>(&mut self, name: &str, val: T) -> error::Result<Attribute>
    where
        T: Into<AttrValue>,
    {
        Attribute::put(self.ncid, self.varid, name, val.into())
    }
}

impl<'g> Variable<'g> {
    fn value_mono<T: NcPutGet>(&self, extent: &Extents) -> error::Result<T> {
        let dims = self.dimensions();
        let (start, count, _stride) = extent.get_start_count_stride(dims)?;

        let number_of_items = count.iter().copied().fold(1_usize, usize::saturating_mul);
        if number_of_items != 1 {
            return Err(error::Error::BufferLen {
                wanted: 1,
                actual: number_of_items,
            });
        }

        unsafe { T::get_var1(self, &start) }
    }

    ///  Fetches one specific value at specific indices
    ///  indices must has the same length as self.dimensions.
    pub fn value<T: NcPutGet, E>(&self, indices: E) -> error::Result<T>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let extent = indices.try_into().map_err(Into::into)?;
        self.value_mono(&extent)
    }

    fn string_value_mono(&self, extent: &Extents) -> error::Result<String> {
        let dims = self.dimensions();
        let (start, count, _stride) = extent.get_start_count_stride(dims)?;

        let number_of_items = count.iter().copied().fold(1_usize, usize::saturating_mul);
        if number_of_items != 1 {
            return Err(error::Error::BufferLen {
                wanted: 1,
                actual: number_of_items,
            });
        }

        let mut s: *mut std::os::raw::c_char = std::ptr::null_mut();
        unsafe {
            error::checked(super::with_lock(|| {
                nc_get_var1_string(self.ncid, self.varid, start.as_ptr(), &mut s)
            }))?;
        }
        let string = unsafe { NcString::from_ptr(s) };
        Ok(string.to_string_lossy().into_owned())
    }

    /// Reads a string variable. This involves two copies per read, and should
    /// be avoided in performance critical code
    pub fn string_value<E>(&self, indices: E) -> error::Result<String>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let extent = indices.try_into().map_err(Into::into)?;
        self.string_value_mono(&extent)
    }

    fn values_mono<T: NcPutGet>(&self, extents: &Extents) -> error::Result<Vec<T>> {
        let dims = self.dimensions();
        let (start, count, stride) = extents.get_start_count_stride(dims)?;

        let number_of_elements = count.iter().copied().fold(1_usize, usize::saturating_mul);
        let mut values = Vec::with_capacity(number_of_elements);

        unsafe {
            T::get_vars(self, &start, &count, &stride, values.as_mut_ptr())?;
            values.set_len(number_of_elements);
        };
        Ok(values)
    }

    /// Get multiple values from a variable
    pub fn values<T: NcPutGet, E>(&self, extents: E) -> error::Result<Vec<T>>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let extents: Extents = extents.try_into().map_err(Into::into)?;
        self.values_mono(&extents)
    }

    #[cfg(feature = "ndarray")]
    /// Fetches variable
    fn values_arr_mono<T: NcPutGet>(&self, extents: &Extents) -> error::Result<ArrayD<T>> {
        let dims = self.dimensions();
        let (start, count, stride) = extents.get_start_count_stride(dims)?;

        let number_of_elements = count.iter().copied().fold(1_usize, usize::saturating_mul);
        let mut values = Vec::with_capacity(number_of_elements);
        unsafe {
            T::get_vars(self, &start, &count, &stride, values.as_mut_ptr())?;
            values.set_len(number_of_elements);
        };

        Ok(ArrayD::from_shape_vec(count, values).unwrap())
    }

    #[cfg(feature = "ndarray")]
    /// Fetches variable
    pub fn values_arr<T: NcPutGet, E>(&self, extents: E) -> error::Result<ArrayD<T>>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let extents: Extents = extents.try_into().map_err(Into::into)?;
        self.values_arr_mono(&extents)
    }

    /// Get the fill value of a variable
    pub fn fill_value<T: NcPutGet>(&self) -> error::Result<Option<T>> {
        if T::NCTYPE != self.vartype {
            return Err(error::Error::TypeMismatch);
        }
        let mut location = std::mem::MaybeUninit::uninit();
        let mut nofill: nc_type = 0;
        unsafe {
            error::checked(super::with_lock(|| {
                nc_inq_var_fill(
                    self.ncid,
                    self.varid,
                    &mut nofill,
                    std::ptr::addr_of_mut!(location).cast(),
                )
            }))?;
        }
        if nofill == 1 {
            return Ok(None);
        }

        Ok(Some(unsafe { location.assume_init() }))
    }

    fn values_to_mono<T: NcPutGet>(
        &self,
        buffer: &mut [T],
        extents: &Extents,
    ) -> error::Result<()> {
        let dims = self.dimensions();
        let (start, count, stride) = extents.get_start_count_stride(dims)?;

        let number_of_elements = count.iter().copied().fold(1_usize, usize::saturating_mul);
        if number_of_elements != buffer.len() {
            return Err(error::Error::BufferLen {
                wanted: number_of_elements,
                actual: buffer.len(),
            });
        }
        unsafe { T::get_vars(self, &start, &count, &stride, buffer.as_mut_ptr()) }
    }
    /// Fetches variable into slice
    /// buffer must be able to hold all the requested elements
    pub fn values_to<T: NcPutGet, E>(&self, buffer: &mut [T], extents: E) -> error::Result<()>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let extents: Extents = extents.try_into().map_err(Into::into)?;
        self.values_to_mono(buffer, &extents)
    }

    fn raw_values_mono(&self, buf: &mut [u8], extents: &Extents) -> error::Result<()> {
        let dims = self.dimensions();
        let (start, count, stride) = extents.get_start_count_stride(dims)?;

        let typ = self.vartype();
        match typ {
            super::types::VariableType::String | super::types::VariableType::Vlen(_) => {
                return Err(error::Error::TypeMismatch)
            }
            _ => (),
        }

        let number_of_elements = count.iter().copied().fold(1_usize, usize::saturating_mul);
        let bytes_requested = number_of_elements.saturating_mul(typ.size());
        if buf.len() != bytes_requested {
            return Err(error::Error::BufferLen {
                wanted: buf.len(),
                actual: bytes_requested,
            });
        }

        error::checked(super::with_lock(|| unsafe {
            nc_get_vars(
                self.ncid,
                self.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                buf.as_mut_ptr().cast(),
            )
        }))
    }
    /// Get values of any type as bytes, with no further interpretation
    /// of the values.
    ///
    /// # Note
    ///
    /// When working with compound types, variable length arrays and
    /// strings will be allocated in `buf`, and this library will
    /// not keep track of the allocations.
    /// This can lead to memory leaks.
    pub fn raw_values<E>(&self, buf: &mut [u8], extents: E) -> error::Result<()>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let extents: Extents = extents.try_into().map_err(Into::into)?;
        self.raw_values_mono(buf, &extents)
    }

    fn vlen_mono<T: NcPutGet>(&self, extent: &Extents) -> error::Result<Vec<T>> {
        let dims = self.dimensions();
        let (start, count, _stride) = extent.get_start_count_stride(dims)?;

        let number_of_items = count.iter().copied().fold(1_usize, usize::saturating_mul);
        if number_of_items != 1 {
            return Err(error::Error::BufferLen {
                wanted: 1,
                actual: number_of_items,
            });
        }
        if let super::types::VariableType::Vlen(v) = self.vartype() {
            if v.typ().id() != T::NCTYPE {
                return Err(error::Error::TypeMismatch);
            }
        } else {
            return Err(error::Error::TypeMismatch);
        };

        let mut vlen: MaybeUninit<nc_vlen_t> = MaybeUninit::uninit();

        error::checked(super::with_lock(|| unsafe {
            nc_get_vara(
                self.ncid,
                self.varid,
                start.as_ptr(),
                count.as_ptr(),
                vlen.as_mut_ptr().cast(),
            )
        }))?;

        let mut vlen = unsafe { vlen.assume_init() };

        let mut v = Vec::<T>::with_capacity(vlen.len);

        unsafe {
            std::ptr::copy_nonoverlapping(vlen.p as *const T, v.as_mut_ptr(), vlen.len);
            v.set_len(vlen.len);
        }
        error::checked(super::with_lock(|| unsafe { nc_free_vlen(&mut vlen) })).unwrap();

        Ok(v)
    }
    /// Get a vlen element
    pub fn vlen<T: NcPutGet, E>(&self, indices: E) -> error::Result<Vec<T>>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let extent = indices.try_into().map_err(Into::into)?;
        self.vlen_mono(&extent)
    }
}

impl<'g> VariableMut<'g> {
    fn put_value_mono<T: NcPutGet>(&mut self, value: T, extents: &Extents) -> error::Result<()> {
        let dims = self.dimensions();
        let (start, count, _stride) = extents.get_start_count_stride(dims)?;

        let number_of_items = count.iter().copied().fold(1_usize, usize::saturating_mul);
        if number_of_items != 1 {
            return Err(error::Error::BufferLen {
                wanted: 1,
                actual: number_of_items,
            });
        }

        unsafe { T::put_var1(self, &start, value) }
    }
    /// Put a single value at `indices`
    pub fn put_value<T: NcPutGet, E>(&mut self, value: T, extents: E) -> error::Result<()>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let extents: Extents = extents.try_into().map_err(Into::into)?;
        self.put_value_mono(value, &extents)
    }

    fn put_string_mono(&mut self, value: &str, extent: &Extents) -> error::Result<()> {
        let dims = self.dimensions();
        let (start, count, _stride) = extent.get_start_count_stride(dims)?;

        let number_of_items = count.iter().copied().fold(1_usize, usize::saturating_mul);
        if number_of_items != 1 {
            return Err(error::Error::BufferLen {
                wanted: 1,
                actual: number_of_items,
            });
        }

        let value = std::ffi::CString::new(value).expect("String contained interior 0");
        let mut ptr = value.as_ptr();

        unsafe {
            error::checked(super::with_lock(|| {
                nc_put_var1_string(self.ncid, self.varid, start.as_ptr(), &mut ptr)
            }))?;
        }

        Ok(())
    }
    /// Internally converts to a `CString`, avoid using this function when performance
    /// is important
    pub fn put_string<E>(&mut self, value: &str, extent: E) -> error::Result<()>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let extents: Extents = extent.try_into().map_err(Into::into)?;
        self.put_string_mono(value, &extents)
    }

    fn put_values_mono<T: NcPutGet>(
        &mut self,
        values: &[T],
        extents: &Extents,
    ) -> error::Result<()> {
        let dims = self.dimensions();
        let (start, mut count, stride) = extents.get_start_count_stride(dims)?;

        let number_of_elements_to_put = values.len();
        let number_of_elements = count.iter().copied().fold(1_usize, usize::saturating_mul);
        if number_of_elements != number_of_elements_to_put {
            if dims.len() == 1 {
                count[0] = values.len();
            } else {
                return Err(error::Error::BufferLen {
                    wanted: number_of_elements,
                    actual: number_of_elements_to_put,
                });
            }
        }

        unsafe {
            T::put_vars(self, &start, &count, &stride, values.as_ptr())?;
        };
        Ok(())
    }
    /// Put a slice of values at `indices`
    pub fn put_values<T: NcPutGet, E>(&mut self, values: &[T], extents: E) -> error::Result<()>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let extents: Extents = extents.try_into().map_err(Into::into)?;
        self.put_values_mono(values, &extents)
    }

    /// Set a Fill Value
    ///
    /// # Errors
    ///
    /// Not a `netCDF-4` file, late define, `fill_value` has the wrong type
    #[allow(clippy::needless_pass_by_value)] // All values will be small
    pub fn set_fill_value<T>(&mut self, fill_value: T) -> error::Result<()>
    where
        T: NcPutGet,
    {
        if T::NCTYPE != self.vartype {
            return Err(error::Error::TypeMismatch);
        }
        unsafe {
            error::checked(super::with_lock(|| {
                nc_def_var_fill(
                    self.ncid,
                    self.varid,
                    NC_FILL,
                    std::ptr::addr_of!(fill_value).cast(),
                )
            }))?;
        }
        Ok(())
    }

    /// Set the fill value to no value. Use this when wanting to avoid
    /// duplicate writes into empty variables.
    ///
    /// # Errors
    ///
    /// Not a `netCDF-4` file
    ///
    /// # Safety
    ///
    /// Reading from this variable after having defined nofill
    /// will read potentially uninitialized data. Normally
    /// one will expect to find some filler value
    pub unsafe fn set_nofill(&mut self) -> error::Result<()> {
        error::checked(super::with_lock(|| {
            nc_def_var_fill(self.ncid, self.varid, NC_NOFILL, std::ptr::null_mut())
        }))
    }

    /// Set endianness of the variable. Must be set before inserting data
    ///
    /// `endian` can take a `Endianness` value with Native being `NC_ENDIAN_NATIVE` (0),
    /// Little `NC_ENDIAN_LITTLE` (1), Big `NC_ENDIAN_BIG` (2)
    ///
    /// # Errors
    ///
    /// Not a `netCDF-4` file, late define
    pub fn endian(&mut self, e: Endianness) -> error::Result<()> {
        let endianness = match e {
            Endianness::Native => NC_ENDIAN_NATIVE,
            Endianness::Little => NC_ENDIAN_LITTLE,
            Endianness::Big => NC_ENDIAN_BIG,
        };
        unsafe {
            error::checked(super::with_lock(|| {
                nc_def_var_endian(self.ncid, self.varid, endianness)
            }))?;
        }
        Ok(())
    }

    unsafe fn put_raw_values_mono(&mut self, buf: &[u8], extents: &Extents) -> error::Result<()> {
        let dims = self.dimensions();
        let (start, count, stride) = extents.get_start_count_stride(dims)?;

        let typ = self.vartype();
        match typ {
            super::types::VariableType::String | super::types::VariableType::Vlen(_) => {
                return Err(error::Error::TypeMismatch)
            }
            _ => (),
        }

        let number_of_elements = count.iter().copied().fold(1_usize, usize::saturating_mul);
        let bytes_requested = number_of_elements.saturating_mul(typ.size());
        if buf.len() != bytes_requested {
            return Err(error::Error::BufferLen {
                wanted: buf.len(),
                actual: bytes_requested,
            });
        }

        #[allow(unused_unsafe)]
        error::checked(super::with_lock(|| unsafe {
            nc_put_vars(
                self.ncid,
                self.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                buf.as_ptr().cast(),
            )
        }))
    }
    /// Get values of any type as bytes
    ///
    /// # Safety
    ///
    /// When working with compound types, variable length arrays and
    /// strings create pointers from the buffer, and tries to copy
    /// memory from these locations. Compound types which does not
    /// have these elements will be safe to access, and can treat
    /// this function as safe
    pub unsafe fn put_raw_values<E>(&mut self, buf: &[u8], extents: E) -> error::Result<()>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let extents: Extents = extents.try_into().map_err(Into::into)?;
        self.put_raw_values_mono(buf, &extents)
    }

    fn put_vlen_mono<T: NcPutGet>(&mut self, vec: &[T], extent: &Extents) -> error::Result<()> {
        let dims = self.dimensions();
        let (start, count, stride) = extent.get_start_count_stride(dims)?;

        let number_of_items = count.iter().copied().fold(1_usize, usize::saturating_mul);
        if number_of_items != 1 {
            return Err(error::Error::BufferLen {
                wanted: 1,
                actual: number_of_items,
            });
        }

        if let super::types::VariableType::Vlen(v) = self.vartype() {
            if v.typ().id() != T::NCTYPE {
                return Err(error::Error::TypeMismatch);
            }
        } else {
            return Err(error::Error::TypeMismatch);
        };

        let vlen = nc_vlen_t {
            len: vec.len(),
            p: vec.as_ptr() as *mut _,
        };

        error::checked(super::with_lock(|| unsafe {
            nc_put_vars(
                self.ncid,
                self.varid,
                start.as_ptr(),
                count.as_ptr(),
                stride.as_ptr(),
                std::ptr::addr_of!(vlen).cast(),
            )
        }))
    }
    /// Get a vlen element
    pub fn put_vlen<T: NcPutGet, E>(&mut self, vec: &[T], indices: E) -> error::Result<()>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let extent = indices.try_into().map_err(Into::into)?;
        self.put_vlen_mono(vec, &extent)
    }
}

impl<'g> VariableMut<'g> {
    pub(crate) fn add_from_str(
        ncid: nc_type,
        xtype: nc_type,
        name: &str,
        dims: &[&str],
    ) -> error::Result<Self> {
        let dimensions = dims
            .iter()
            .map(
                |dimname| match super::dimension::from_name_toid(ncid, dimname) {
                    Ok(Some(id)) => Ok(id),
                    Ok(None) => Err(error::Error::NotFound(format!("dimensions {dimname}"))),
                    Err(e) => Err(e),
                },
            )
            .collect::<error::Result<Vec<_>>>()?;

        let cname = super::utils::short_name_to_bytes(name)?;
        let mut varid = 0;
        unsafe {
            let dimlen = dimensions.len().try_into()?;
            error::checked(super::with_lock(|| {
                nc_def_var(
                    ncid,
                    cname.as_ptr().cast(),
                    xtype,
                    dimlen,
                    dimensions.as_ptr(),
                    &mut varid,
                )
            }))?;
        }

        let dimensions = dims
            .iter()
            .map(|dimname| match super::dimension::from_name(ncid, dimname) {
                Ok(None) => Err(error::Error::NotFound(format!("dimensions {dimname}"))),
                Ok(Some(dim)) => Ok(dim),
                Err(e) => Err(e),
            })
            .collect::<error::Result<Vec<_>>>()?;

        Ok(VariableMut(
            Variable {
                ncid,
                varid,
                vartype: xtype,
                dimensions,
                _group: PhantomData,
            },
            PhantomData,
        ))
    }
}

pub(crate) fn variables_at_ncid<'g>(
    ncid: nc_type,
) -> error::Result<impl Iterator<Item = error::Result<Variable<'g>>>> {
    let mut nvars = 0;
    unsafe {
        error::checked(super::with_lock(|| {
            nc_inq_varids(ncid, &mut nvars, std::ptr::null_mut())
        }))?;
    }
    let mut varids = vec![0; nvars.try_into()?];
    unsafe {
        error::checked(super::with_lock(|| {
            nc_inq_varids(ncid, std::ptr::null_mut(), varids.as_mut_ptr())
        }))?;
    }
    Ok(varids.into_iter().map(move |varid| {
        let mut xtype = 0;
        unsafe {
            error::checked(super::with_lock(|| nc_inq_vartype(ncid, varid, &mut xtype)))?;
        }
        let dimensions = super::dimension::dimensions_from_variable(ncid, varid)?
            .collect::<error::Result<Vec<_>>>()?;
        Ok(Variable {
            ncid,
            varid,
            dimensions,
            vartype: xtype,
            _group: PhantomData,
        })
    }))
}

pub(crate) fn add_variable_from_identifiers<'g>(
    ncid: nc_type,
    name: &str,
    dims: &[super::dimension::Identifier],
    xtype: nc_type,
) -> error::Result<VariableMut<'g>> {
    let cname = super::utils::short_name_to_bytes(name)?;

    let dimensions = dims
        .iter()
        .map(move |&id| {
            // Internal netcdf detail, the top 16 bits gives the corresponding
            // file handle. This to ensure dimensions are not added from another
            // file which is unrelated to self
            if id.ncid >> 16 != ncid >> 16 {
                return Err(error::Error::WrongDataset);
            }
            let mut dimlen = 0;
            unsafe {
                error::checked(super::with_lock(|| {
                    nc_inq_dimlen(id.ncid, id.dimid, &mut dimlen)
                }))?;
            }
            Ok(Dimension {
                len: core::num::NonZeroUsize::new(dimlen),
                id,
                _group: PhantomData,
            })
        })
        .collect::<error::Result<Vec<_>>>()?;
    let dims = dims.iter().map(|x| x.dimid).collect::<Vec<_>>();

    let mut varid = 0;
    unsafe {
        let dimlen = dims.len().try_into()?;
        error::checked(super::with_lock(|| {
            nc_def_var(
                ncid,
                cname.as_ptr().cast(),
                xtype,
                dimlen,
                dims.as_ptr(),
                &mut varid,
            )
        }))?;
    }

    Ok(VariableMut(
        Variable {
            ncid,
            dimensions,
            varid,
            vartype: xtype,
            _group: PhantomData,
        },
        PhantomData,
    ))
}
