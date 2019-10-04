use super::attribute::AttrValue;
use super::attribute::Attribute;
use super::dimension::Dimension;
use super::error;
use super::HashMap;
use super::LOCK;
#[cfg(feature = "ndarray")]
use ndarray::ArrayD;
use netcdf_sys::*;
use std::marker::Sized;

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

impl Variable {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn attributes(&self) -> &HashMap<String, Attribute> {
        &self.attributes
    }
    pub fn dimensions(&self) -> &[Dimension] {
        &self.dimensions
    }
    pub fn vartype(&self) -> nc_type {
        self.vartype
    }
    /// Sets compression on the variable. Must be set before filling in data
    pub fn compression(
        &mut self,
        deflate_level: nc_type,
        chunksize: Option<usize>,
    ) -> error::Result<()> {
        let _l = LOCK.lock().unwrap();
        if let Some(chunks) = chunksize {
            unsafe {
                error::checked(nc_def_var_chunking(
                    self.ncid, self.varid, NC_CHUNKED, &chunks,
                ))?;
            }
        }
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
            if i + s > d.len() {
                if !putting {
                    return Err(error::Error::SliceMismatch);
                }
                if !d.is_unlimited() {
                    return Err(error::Error::SliceMismatch);
                }
            }
        }

        let thislen = sizelen.iter().fold(1, |acc, x| acc * x);
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
            } else {
                if putting && d.is_unlimited() {
                    unlim_pos = match unlim_pos {
                        Some(_) => return Err(error::Error::Ambiguous),
                        None => {
                            if d.is_unlimited() {
                                Some(pos)
                            } else {
                                return Err(error::Error::SliceMismatch);
                            }
                        }
                    };
                    sizelen.push(1);
                } else {
                    sizelen.push(d.len() - i);
                }
            }
        }

        if let Some(pos) = unlim_pos {
            let l = sizelen.iter().fold(1, |acc, x| acc * x);
            sizelen[pos] = totallen / l;
        }

        let wantedlen = sizelen.iter().fold(1, |acc, x| acc * x);
        if totallen != wantedlen {
            return Err(error::Error::BufferLen(totallen, wantedlen));
        }
        Ok(sizelen)
    }
}

/// This trait allow an implicit cast when fetching
/// a netCDF variable
pub trait Numeric
where
    Self: Sized,
{
    const NCTYPE: nc_type;
    /// Returns a single indexed value of the variable as Self
    fn single_value_from_variable(variable: &Variable, indices: &[usize]) -> error::Result<Self>;

    /// Get multiple values at once
    unsafe fn variable_to_ptr(
        variable: &Variable,
        indices: &[usize],
        slice_len: &[usize],
        values: *mut Self,
    ) -> error::Result<()>;

    /// Put a single value into a netCDF variable
    fn put_value_at(variable: &mut Variable, indices: &[usize], value: Self) -> error::Result<()>;

    /// put a SLICE of values into a netCDF variable at the given index
    fn put_values_at(
        variable: &mut Variable,
        indices: &[usize],
        slice_len: &[usize],
        values: &[Self],
    ) -> error::Result<()>;
}

// This macro implements the trait Numeric for the type "sized_type".
// The use of this macro reduce code duplication for the implementation of Numeric
// for the common numeric types (i32, f32 ...): they only differs by the name of the
// C function used to fetch values from the NetCDF variable (eg: 'nc_get_var_ushort', ...).
//
macro_rules! impl_numeric {
    (
        $sized_type: ty,
        $nc_type: ident,
        $nc_get_var: ident,
        $nc_get_vara_type: ident,
        $nc_get_var1_type: ident,
        $nc_put_var1_type: ident,
        $nc_put_vara_type: ident) => {
        impl Numeric for $sized_type {
            const NCTYPE: nc_type = $nc_type;
            // fetch ONE value from variable using `$nc_get_var1`
            fn single_value_from_variable(
                variable: &Variable,
                indices: &[usize],
            ) -> error::Result<$sized_type> {
                // initialize `buff` to 0
                let mut buff: $sized_type = 0 as $sized_type;
                // Get a pointer to an array
                let indices_ptr = indices.as_ptr();
                unsafe {
                    let _g = LOCK.lock().unwrap();
                    error::checked($nc_get_var1_type(
                        variable.ncid,
                        variable.varid,
                        indices_ptr,
                        &mut buff,
                    ))?;
                }
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
            fn put_value_at(
                variable: &mut Variable,
                indices: &[usize],
                value: Self,
            ) -> error::Result<()> {
                unsafe {
                    let _g = LOCK.lock().unwrap();
                    error::checked($nc_put_var1_type(
                        variable.ncid,
                        variable.varid,
                        indices.as_ptr(),
                        &value,
                    ))
                }
            }

            // put a SLICE of values into a netCDF variable at the given index
            fn put_values_at(
                variable: &mut Variable,
                indices: &[usize],
                slice_len: &[usize],
                values: &[Self],
            ) -> error::Result<()> {
                unsafe {
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
        dims: &[&Dimension],
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
                dimids.len() as _,
                dimids.as_ptr(),
                &mut id,
            ))?;
        }

        Ok(Self {
            name: name.into(),
            attributes: HashMap::new(),
            dimensions: dims.iter().map(|x| (**x).clone()).collect(),
            vartype,
            ncid: grp_id,
            varid: id,
        })
    }

    pub fn add_attribute<T>(&mut self, name: &str, val: T) -> error::Result<()>
    where
        T: Into<AttrValue>,
    {
        let att = Attribute::put(self.ncid, self.varid, name, val.into())?;
        self.attributes.insert(name.to_string().clone(), att);
        Ok(())
    }

    ///  Fetches one specific value at specific indices
    ///  indices must has the same length as self.dimensions.
    pub fn value<T: Numeric>(&self, indices: Option<&[usize]>) -> error::Result<T> {
        let _indices: Vec<usize>;
        let indices = match indices {
            Some(x) => {
                self.check_indices(x, false)?;
                x
            }
            None => {
                _indices = self.default_indices(false)?;
                &_indices
            }
        };

        T::single_value_from_variable(self, indices)
    }

    #[cfg(feature = "ndarray")]
    /// Fetches variable
    pub fn values<T: Numeric>(
        &self,
        indices: Option<&[usize]>,
        size_len: Option<&[usize]>,
    ) -> error::Result<ArrayD<T>> {
        let _indices: Vec<usize>;
        let indices = match indices {
            Some(x) => {
                self.check_indices(x, false)?;
                x
            }
            None => {
                _indices = self.default_indices(false)?;
                &_indices
            }
        };
        let _size_len: Vec<usize>;
        let full_length = self.dimensions.iter().fold(1, |acc, d| acc * d.len());
        let size_len = match size_len {
            Some(x) => {
                self.check_sizelen(full_length, indices, x, false)?;
                x
            }
            None => {
                _size_len = self.default_sizelen(full_length, indices, false)?;
                &_size_len
            }
        };

        let mut values = Vec::with_capacity(full_length);
        unsafe {
            T::variable_to_ptr(self, indices, size_len, values.as_mut_ptr())?;
            values.set_len(full_length);
        }
        Ok(ArrayD::from_shape_vec(size_len, values).unwrap())
    }

    /// Fetches variable into slice
    /// buffer must be able to hold all the requested elements
    pub fn values_to<T: Numeric>(
        &self,
        buffer: &mut [T],
        indices: Option<&[usize]>,
        size_len: Option<&[usize]>,
    ) -> error::Result<()> {
        let _indices: Vec<usize>;
        let indices = match indices {
            Some(x) => {
                self.check_indices(x, false)?;
                x
            }
            None => {
                _indices = self.default_indices(false)?;
                &_indices
            }
        };
        let _size_len: Vec<usize>;
        let size_len = match size_len {
            Some(x) => {
                self.check_sizelen(buffer.len(), indices, x, false)?;
                x
            }
            None => {
                _size_len = self.default_sizelen(buffer.len(), indices, false)?;
                &_size_len
            }
        };

        unsafe { T::variable_to_ptr(self, indices, size_len, buffer.as_mut_ptr()) }
    }

    /// Put a single value at `indices`
    pub fn put_value<T: Numeric>(
        &mut self,
        value: T,
        indices: Option<&[usize]>,
    ) -> error::Result<()> {
        let _indices: Vec<usize>;
        let indices = match indices {
            Some(x) => {
                self.check_indices(x, true)?;
                x
            }
            None => {
                _indices = self.default_indices(true)?;
                &_indices
            }
        };
        T::put_value_at(self, indices, value)
    }

    /// Put a slice of values at `indices`
    pub fn put_values<T: Numeric>(
        &mut self,
        values: &[T],
        indices: Option<&[usize]>,
        slice_len: Option<&[usize]>,
    ) -> error::Result<()> {
        let _indices: Vec<usize>;
        let indices = match indices {
            Some(x) => {
                self.check_indices(x, true)?;
                x
            }
            None => {
                _indices = self.default_indices(true)?;
                &_indices
            }
        };
        let _size_len: Vec<usize>;
        let slice_len = match slice_len {
            Some(x) => {
                self.check_sizelen(values.len(), indices, x, true)?;
                x
            }
            None => {
                _size_len = self.default_sizelen(values.len(), indices, true)?;
                &_size_len
            }
        };
        T::put_values_at(self, indices, slice_len, values)
    }

    /// Set a Fill Value
    pub fn set_fill_value<T>(&mut self, fill_value: T) -> error::Result<()>
    where
        T: Numeric + Into<super::attribute::AttrValue>,
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
