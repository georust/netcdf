//! Variables in the netcdf file

#![allow(clippy::similar_names)]
use super::attribute::AttrValue;
use super::attribute::Attribute;
use super::dimension::Dimension;
use super::error;
use super::HashMap;
use super::LOCK;
#[cfg(feature = "ndarray")]
use ndarray::ArrayD;
use netcdf_sys::*;
use std::convert::TryInto;
use std::marker::Sized;

#[allow(clippy::doc_markdown)]
/// This struct defines a netCDF variable.
#[derive(Debug)]
pub struct Variable {
    /// The variable name
    pub(crate) name: String,
    pub(crate) attributes: HashMap<String, Attribute>,
    pub(crate) dimensions: Vec<Dimension>,
    /// the netcdf variable type identifier (from netcdf-sys)
    pub(crate) vartype: nc_type,
    pub(crate) ncid: nc_type,
    pub(crate) varid: nc_type,
}

#[allow(clippy::len_without_is_empty)]
impl Variable {
    /// Get name of variable
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Get an attribute of this variable
    pub fn attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.get(name)
    }
    /// Iterator over all the attributes of this variable
    pub fn attributes(&self) -> impl Iterator<Item = &Attribute> {
        self.attributes.values()
    }
    /// Dimensions for a variable
    pub fn dimensions(&self) -> &[Dimension] {
        &self.dimensions
    }
    /// Get the type of this variable. This will be an integer
    /// such as `NC_FLOAT`, `NC_DOUBLE`, `NC_INT` from
    /// the `netcdf-sys` crate
    pub fn vartype(&self) -> nc_type {
        self.vartype
    }
    /// Get current length of the variable
    pub fn len(&self) -> usize {
        self.dimensions.iter().map(Dimension::len).product()
    }

    /// Sets compression on the variable. Must be set before filling in data.
    ///
    /// `deflate_level` can take a value 0..=9, with 0 being no
    /// compression (good for CPU bound tasks), and 9 providing the
    /// highest compression level (good for memory bound tasks)
    pub fn compression(&mut self, deflate_level: nc_type) -> error::Result<()> {
        let _l = LOCK.lock().unwrap();
        unsafe {
            error::checked(nc_def_var_deflate(
                self.ncid,
                self.varid,
                false as _,
                true as _,
                deflate_level,
            ))?;
        }

        Ok(())
    }

    /// Set chunking for variable. Must be set before inserting data
    ///
    /// Use this when reading or writing smaller units of the hypercube than
    /// the full dimensions lengths, to change how the variable is stored in
    /// the file. This has no effect on the memory order when reading/putting
    /// a buffer.
    pub fn chunking(&mut self, chunksize: &[usize]) -> error::Result<()> {
        let _l = LOCK.lock().unwrap();
        if chunksize.len() != self.dimensions.len() {
            return Err(error::Error::SliceLen);
        }
        let len = chunksize
            .iter()
            .fold(1_usize, |acc, &x| acc.saturating_mul(x));
        if len == usize::max_value() {
            return Err(error::Error::Overflow);
        }
        unsafe {
            error::checked(nc_def_var_chunking(
                self.ncid,
                self.varid,
                NC_CHUNKED,
                chunksize.as_ptr(),
            ))?;
        }

        Ok(())
    }

    /// Checks for array mismatch
    fn check_indices(&self, indices: &[usize], putting: bool) -> error::Result<()> {
        if indices.len() != self.dimensions.len() {
            return Err(error::Error::IndexLen);
        }

        for (d, i) in self.dimensions.iter().zip(indices) {
            if d.is_unlimited() && putting {
                continue;
            }
            if *i > d.len() {
                return Err(error::Error::IndexMismatch);
            }
        }

        Ok(())
    }
    /// Create a default [0, 0, ..., 0] offset
    fn default_indices(&self, putting: bool) -> error::Result<Vec<usize>> {
        self.dimensions
            .iter()
            .map(|d| {
                if d.len() > 0 || putting {
                    Ok(0)
                } else {
                    Err(error::Error::IndexMismatch)
                }
            })
            .collect()
    }

    /// Assumes indices is valid for this variable
    fn check_sizelen(
        &self,
        totallen: usize,
        indices: &[usize],
        sizelen: &[usize],
        putting: bool,
    ) -> error::Result<()> {
        if sizelen.len() != self.dimensions.len() {
            return Err(error::Error::SliceLen);
        }

        for ((i, s), d) in indices.iter().zip(sizelen).zip(&self.dimensions) {
            if *s == 0 {
                return Err(error::Error::ZeroSlice);
            }
            if i.checked_add(*s).is_none() {
                return Err(error::Error::Overflow);
            }
            if i + s > d.len() {
                if !putting {
                    return Err(error::Error::SliceMismatch);
                }
                if !d.is_unlimited() {
                    return Err(error::Error::SliceMismatch);
                }
            }
        }

        let thislen = sizelen
            .iter()
            .fold(1_usize, |acc, &x| acc.saturating_mul(x));
        if thislen == usize::max_value() {
            return Err(error::Error::Overflow);
        }

        if totallen != thislen {
            return Err(error::Error::BufferLen(totallen, thislen));
        }

        Ok(())
    }

    /// Assumes indices is valid for this variable
    fn default_sizelen(
        &self,
        totallen: usize,
        indices: &[usize],
        putting: bool,
    ) -> error::Result<Vec<usize>> {
        let num_unlims = self
            .dimensions
            .iter()
            .fold(0, |acc, x| acc + x.is_unlimited() as usize);
        if num_unlims > 1 {
            return Err(error::Error::Ambiguous);
        }

        let mut sizelen = Vec::with_capacity(self.dimensions.len());

        let mut unlim_pos = None;
        for (pos, (&i, d)) in indices.iter().zip(&self.dimensions).enumerate() {
            if i >= d.len() {
                if !d.is_unlimited() {
                    return Err(error::Error::SliceMismatch);
                }
                if !putting {
                    return Err(error::Error::SliceMismatch);
                }
                unlim_pos = Some(pos);
                sizelen.push(1);
            } else if putting && d.is_unlimited() {
                unlim_pos = Some(pos);
                sizelen.push(1);
            } else {
                sizelen.push(d.len() - i);
            }
        }

        if let Some(pos) = unlim_pos {
            let l = sizelen
                .iter()
                .fold(1_usize, |acc, &x| acc.saturating_mul(x));
            if l == usize::max_value() {
                return Err(error::Error::Overflow);
            }
            sizelen[pos] = totallen / l;
        }

        let wantedlen = sizelen
            .iter()
            .fold(1_usize, |acc, &x| acc.saturating_mul(x));
        if wantedlen == usize::max_value() {
            return Err(error::Error::Overflow);
        }
        if totallen != wantedlen {
            return Err(error::Error::BufferLen(totallen, wantedlen));
        }
        Ok(sizelen)
    }
}

#[allow(clippy::doc_markdown)]
/// This trait allow an implicit cast when fetching
/// a netCDF variable. These methods are not be called
/// directly, but used through methods on `Variable`
pub unsafe trait Numeric
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
    unsafe fn single_value_from_variable(
        variable: &Variable,
        indices: &[usize],
    ) -> error::Result<Self>;

    /// Get multiple values at once, without checking the validity of
    /// `indices` or `slice_len`
    ///
    /// # Safety
    ///
    /// Requires `values` to be of at least size `slice_len.product()`,
    /// `indices` and `slice_len` to be of a valid length
    unsafe fn variable_to_ptr(
        variable: &Variable,
        indices: &[usize],
        slice_len: &[usize],
        values: *mut Self,
    ) -> error::Result<()>;

    #[allow(clippy::doc_markdown)]
    /// Put a single value into a netCDF variable
    ///
    /// # Safety
    ///
    /// Requires `indices` to be of a valid length
    unsafe fn put_value_at(
        variable: &mut Variable,
        indices: &[usize],
        value: Self,
    ) -> error::Result<()>;

    #[allow(clippy::doc_markdown)]
    /// put a SLICE of values into a netCDF variable at the given index
    ///
    /// # Safety
    ///
    /// Requires `indices` and `slice_len` to be of a valid length
    unsafe fn put_values_at(
        variable: &mut Variable,
        indices: &[usize],
        slice_len: &[usize],
        values: &[Self],
    ) -> error::Result<()>;
}

#[allow(clippy::doc_markdown)]
/// This macro implements the trait Numeric for the type `sized_type`.
///
/// The use of this macro reduce code duplication for the implementation of Numeric
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
        $nc_put_vara_type: ident) => {
        #[allow(clippy::use_self)] // False positives
        unsafe impl Numeric for $sized_type {
            const NCTYPE: nc_type = $nc_type;

            // fetch ONE value from variable using `$nc_get_var1`
            unsafe fn single_value_from_variable(
                variable: &Variable,
                indices: &[usize],
            ) -> error::Result<Self> {
                // initialize `buff` to 0
                let mut buff: Self = 0 as _;
                // Get a pointer to an array
                let indices_ptr = indices.as_ptr();
                let _g = LOCK.lock().unwrap();
                error::checked($nc_get_var1_type(
                    variable.ncid,
                    variable.varid,
                    indices_ptr,
                    &mut buff,
                ))?;
                Ok(buff)
            }

            unsafe fn variable_to_ptr(
                variable: &Variable,
                indices: &[usize],
                slice_len: &[usize],
                values: *mut Self,
            ) -> error::Result<()> {
                let _l = LOCK.lock().unwrap();

                error::checked($nc_get_vara_type(
                    variable.ncid,
                    variable.varid,
                    indices.as_ptr(),
                    slice_len.as_ptr(),
                    values,
                ))
            }

            // put a SINGLE value into a netCDF variable at the given index
            unsafe fn put_value_at(
                variable: &mut Variable,
                indices: &[usize],
                value: Self,
            ) -> error::Result<()> {
                let _g = LOCK.lock().unwrap();
                error::checked($nc_put_var1_type(
                    variable.ncid,
                    variable.varid,
                    indices.as_ptr(),
                    &value,
                ))
            }

            // put a SLICE of values into a netCDF variable at the given index
            unsafe fn put_values_at(
                variable: &mut Variable,
                indices: &[usize],
                slice_len: &[usize],
                values: &[Self],
            ) -> error::Result<()> {
                let _l = LOCK.lock().unwrap();
                error::checked($nc_put_vara_type(
                    variable.ncid,
                    variable.varid,
                    indices.as_ptr(),
                    slice_len.as_ptr(),
                    values.as_ptr(),
                ))
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
    nc_put_vara_uchar
);

impl_numeric!(
    i8,
    NC_BYTE,
    nc_get_var_schar,
    nc_get_vara_schar,
    nc_get_var1_schar,
    nc_put_var1_schar,
    nc_put_vara_schar
);

impl_numeric!(
    i16,
    NC_SHORT,
    nc_get_var_short,
    nc_get_vara_short,
    nc_get_var1_short,
    nc_put_var1_short,
    nc_put_vara_short
);

impl_numeric!(
    u16,
    NC_USHORT,
    nc_get_var_ushort,
    nc_get_vara_ushort,
    nc_get_var1_ushort,
    nc_put_var1_ushort,
    nc_put_vara_ushort
);

impl_numeric!(
    i32,
    NC_INT,
    nc_get_var_int,
    nc_get_vara_int,
    nc_get_var1_int,
    nc_put_var1_int,
    nc_put_vara_int
);

impl_numeric!(
    u32,
    NC_UINT,
    nc_get_var_uint,
    nc_get_vara_uint,
    nc_get_var1_uint,
    nc_put_var1_uint,
    nc_put_vara_uint
);

impl_numeric!(
    i64,
    NC_INT64,
    nc_get_var_longlong,
    nc_get_vara_longlong,
    nc_get_var1_longlong,
    nc_put_var1_longlong,
    nc_put_vara_longlong
);

impl_numeric!(
    u64,
    NC_UINT64,
    nc_get_var_ulonglong,
    nc_get_vara_ulonglong,
    nc_get_var1_ulonglong,
    nc_put_var1_ulonglong,
    nc_put_vara_ulonglong
);

impl_numeric!(
    f32,
    NC_FLOAT,
    nc_get_var_float,
    nc_get_vara_float,
    nc_get_var1_float,
    nc_put_var1_float,
    nc_put_vara_float
);

impl_numeric!(
    f64,
    NC_DOUBLE,
    nc_get_var_double,
    nc_get_vara_double,
    nc_get_var1_double,
    nc_put_var1_double,
    nc_put_vara_double
);

impl Variable {
    pub(crate) fn new(
        grp_id: nc_type,
        name: &str,
        dims: Vec<Dimension>,
        vartype: nc_type,
    ) -> error::Result<Self> {
        use std::ffi::CString;
        let cname = CString::new(name).unwrap();

        let dimids: Vec<nc_type> = dims.iter().map(|x| x.id).collect();
        let mut id = 0;
        unsafe {
            let _l = LOCK.lock().unwrap();
            error::checked(nc_def_var(
                grp_id,
                cname.as_ptr(),
                vartype,
                dimids.len().try_into()?,
                dimids.as_ptr(),
                &mut id,
            ))?;
        }

        Ok(Self {
            name: name.into(),
            attributes: HashMap::new(),
            dimensions: dims,
            vartype,
            ncid: grp_id,
            varid: id,
        })
    }

    /// Adds an attribute to the variable
    pub fn add_attribute<T>(&mut self, name: &str, val: T) -> error::Result<()>
    where
        T: Into<AttrValue>,
    {
        let att = Attribute::put(self.ncid, self.varid, name, val.into())?;
        self.attributes.insert(name.to_string(), att);
        Ok(())
    }

    ///  Fetches one specific value at specific indices
    ///  indices must has the same length as self.dimensions.
    pub fn value<T: Numeric>(&self, indices: Option<&[usize]>) -> error::Result<T> {
        let indices_: Vec<usize>;
        let indices = if let Some(x) = indices {
            self.check_indices(x, false)?;
            x
        } else {
            indices_ = self.default_indices(false)?;
            &indices_
        };

        unsafe { T::single_value_from_variable(self, indices) }
    }

    /// Reads a string variable. This involves two copies per read, and should
    /// be avoided in performance critical code
    pub fn string_value(&self, indices: Option<&[usize]>) -> error::Result<String> {
        let indices_: Vec<usize>;
        let indices = if let Some(x) = indices {
            self.check_indices(x, false)?;
            x
        } else {
            indices_ = self.default_indices(false)?;
            &indices_
        };
        let _l = LOCK.lock().unwrap();

        let mut s: *mut std::os::raw::c_char = std::ptr::null_mut();
        unsafe {
            error::checked(nc_get_var1_string(
                self.ncid,
                self.varid,
                indices.as_ptr(),
                &mut s,
            ))?;
        }
        let string = unsafe { std::ffi::CStr::from_ptr(s) };

        let value = string.to_string_lossy().into_owned();

        unsafe {
            // Make sure this is always called before exiting function
            error::checked(nc_free_string(1, &mut s))?;
        }

        Ok(value)
    }

    #[cfg(feature = "ndarray")]
    /// Fetches variable
    pub fn values<T: Numeric>(
        &self,
        indices: Option<&[usize]>,
        slice_len: Option<&[usize]>,
    ) -> error::Result<ArrayD<T>> {
        let indices_: Vec<usize>;
        let indices = if let Some(x) = indices {
            self.check_indices(x, false)?;
            x
        } else {
            indices_ = self.default_indices(false)?;
            &indices_
        };
        let slice_len_: Vec<usize>;
        let full_length;
        let slice_len = if let Some(x) = slice_len {
            full_length = x.iter().fold(1_usize, |acc, x| acc.saturating_mul(*x));
            if full_length == usize::max_value() {
                return Err(error::Error::Overflow);
            }
            self.check_sizelen(full_length, indices, x, false)?;
            x
        } else {
            full_length = self.dimensions.iter().map(Dimension::len).product();
            slice_len_ = self.default_sizelen(full_length, indices, false)?;
            &slice_len_
        };

        let mut values = Vec::with_capacity(full_length);
        unsafe {
            T::variable_to_ptr(self, indices, slice_len, values.as_mut_ptr())?;
            values.set_len(full_length);
        }
        Ok(ArrayD::from_shape_vec(slice_len, values).unwrap())
    }

    /// Fetches variable into slice
    /// buffer must be able to hold all the requested elements
    pub fn values_to<T: Numeric>(
        &self,
        buffer: &mut [T],
        indices: Option<&[usize]>,
        slice_len: Option<&[usize]>,
    ) -> error::Result<()> {
        let indices_: Vec<usize>;
        let indices = if let Some(x) = indices {
            self.check_indices(x, false)?;
            x
        } else {
            indices_ = self.default_indices(false)?;
            &indices_
        };
        let slice_len_: Vec<usize>;
        let slice_len = if let Some(x) = slice_len {
            self.check_sizelen(buffer.len(), indices, x, false)?;
            x
        } else {
            slice_len_ = self.default_sizelen(buffer.len(), indices, false)?;
            &slice_len_
        };

        unsafe { T::variable_to_ptr(self, indices, slice_len, buffer.as_mut_ptr()) }
    }

    /// Put a single value at `indices`
    pub fn put_value<T: Numeric>(
        &mut self,
        value: T,
        indices: Option<&[usize]>,
    ) -> error::Result<()> {
        let indices_: Vec<usize>;
        let indices = if let Some(x) = indices {
            self.check_indices(x, true)?;
            x
        } else {
            indices_ = self.default_indices(true)?;
            &indices_
        };
        unsafe { T::put_value_at(self, indices, value) }
    }

    /// Internally converts to a `CString`, avoid using this function when performance
    /// is important
    pub fn put_string(&mut self, value: &str, indices: Option<&[usize]>) -> error::Result<()> {
        let indices_: Vec<usize>;
        let indices = if let Some(x) = indices {
            self.check_indices(x, true)?;
            x
        } else {
            indices_ = self.default_indices(true)?;
            &indices_
        };

        let value = std::ffi::CString::new(value).expect("String contained interior 0");
        let mut ptr = value.as_ptr();

        let _l = LOCK.lock().unwrap();

        unsafe {
            error::checked(nc_put_var1_string(
                self.ncid,
                self.varid,
                indices.as_ptr(),
                &mut ptr,
            ))?
        }

        Ok(())
    }

    /// Put a slice of values at `indices`
    pub fn put_values<T: Numeric>(
        &mut self,
        values: &[T],
        indices: Option<&[usize]>,
        slice_len: Option<&[usize]>,
    ) -> error::Result<()> {
        let indices_: Vec<usize>;
        let indices = if let Some(x) = indices {
            self.check_indices(x, true)?;
            x
        } else {
            indices_ = self.default_indices(true)?;
            &indices_
        };
        let slice_len_: Vec<usize>;
        let slice_len = if let Some(x) = slice_len {
            self.check_sizelen(values.len(), indices, x, true)?;
            x
        } else {
            slice_len_ = self.default_sizelen(values.len(), indices, true)?;
            &slice_len_
        };
        unsafe { T::put_values_at(self, indices, slice_len, values) }
    }

    /// Set a Fill Value
    #[allow(clippy::needless_pass_by_value)] // All values will be small
    pub fn set_fill_value<T>(&mut self, fill_value: T) -> error::Result<()>
    where
        T: Numeric,
    {
        if T::NCTYPE != self.vartype {
            return Err(error::Error::TypeMismatch);
        }
        let _l = LOCK.lock().unwrap();
        unsafe {
            error::checked(nc_def_var_fill(
                self.ncid,
                self.varid,
                NC_FILL,
                &fill_value as *const T as *const _,
            ))?;
        }
        let a = Attribute {
            name: "_FillValue".into(),
            ncid: self.ncid,
            varid: self.varid,
        };
        self.attributes.insert("_FillValue".into(), a);
        Ok(())
    }
    /// Get the fill value of a variable
    pub fn fill_value<T: Numeric>(&self) -> error::Result<Option<T>> {
        if T::NCTYPE != self.vartype {
            return Err(error::Error::TypeMismatch);
        }
        let mut location = std::mem::MaybeUninit::uninit();
        let mut nofill: nc_type = 0;
        let _l = LOCK.lock().unwrap();
        unsafe {
            error::checked(nc_inq_var_fill(
                self.ncid,
                self.varid,
                &mut nofill,
                &mut location as *mut _ as *mut _,
            ))?;
        }
        if nofill == 1 {
            return Ok(None);
        }

        Ok(Some(unsafe { location.assume_init() }))
    }
}
