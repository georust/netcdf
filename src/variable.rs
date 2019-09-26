use super::attribute::AttrValue;
use super::attribute::Attribute;
use super::dimension::Dimension;
use super::error;
use super::LOCK;
#[cfg(feature = "ndarray")]
use ndarray::ArrayD;
use netcdf_sys::*;
use std::collections::HashMap;
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
            let err;
            unsafe {
                err = nc_def_var_chunking(self.ncid, self.varid, NC_CHUNKED, &chunks);
            }
            if err != NC_NOERR {
                return Err(err.into());
            }
        }
        let err;
        unsafe {
            err = nc_def_var_deflate(self.ncid, self.varid, false as _, true as _, deflate_level);
        }
        if err != NC_NOERR {
            return Err(err.into());
        }

        Ok(())
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
    fn single_value_from_variable(
        variable: &Variable,
        indices: Option<&[usize]>,
    ) -> error::Result<Self>;

    #[cfg(feature = "ndarray")]
    /// Returns an ndarray of the variable
    fn array_from_variable(
        variable: &Variable,
        indices: Option<&[usize]>,
        slice_len: Option<&[usize]>,
    ) -> error::Result<ArrayD<Self>>;
    /// Returns a slice of the variable as Vec<Self>
    fn slice_from_variable(
        variable: &Variable,
        indices: Option<&[usize]>,
        slice_len: Option<&[usize]>,
        values: &mut [Self],
    ) -> error::Result<()>;
    /// Put a single value into a netCDF variable
    fn put_value_at(
        variable: &mut Variable,
        indices: Option<&[usize]>,
        value: Self,
    ) -> error::Result<()>;
    /// put a SLICE of values into a netCDF variable at the given index
    fn put_values_at(
        variable: &mut Variable,
        indices: Option<&[usize]>,
        slice_len: Option<&[usize]>,
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
                indices: Option<&[usize]>,
            ) -> error::Result<$sized_type> {
                // Check the length of `indices`
                let _indices: Vec<usize>;
                let indices = match indices {
                    Some(x) => {
                        if x.len() != variable.dimensions.len() {
                            return Err(error::Error::IndexLen);
                        }
                        for i in 0..x.len() {
                            if x[i] >= variable.dimensions[i].len {
                                return Err(error::Error::IndexMismatch);
                            }
                        }
                        x
                    }
                    None => {
                        _indices = variable.dimensions.iter().map(|_| 0).collect();
                        &_indices
                    }
                };
                // initialize `buff` to 0
                let mut buff: $sized_type = 0 as $sized_type;
                let err: nc_type;
                // Get a pointer to an array
                let indices_ptr = indices.as_ptr();
                unsafe {
                    let _g = LOCK.lock().unwrap();
                    err = $nc_get_var1_type(variable.ncid, variable.varid, indices_ptr, &mut buff);
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }
                Ok(buff)
            }

            #[cfg(feature = "ndarray")]
            fn array_from_variable(
                variable: &Variable,
                indices: Option<&[usize]>,
                slice_len: Option<&[usize]>,
            ) -> error::Result<ArrayD<$sized_type>> {
                let _slice_len: Vec<_>;
                let slice_len = match slice_len {
                    Some(x) => x,
                    None => {
                        _slice_len = variable.dimensions.iter().map(|x| x.len).collect();
                        &_slice_len
                    }
                };

                let mut values: ArrayD<$sized_type> = unsafe { ArrayD::uninitialized(slice_len) };

                <$sized_type>::slice_from_variable(
                    variable,
                    indices,
                    Some(slice_len),
                    values.as_slice_mut().ok_or(error::Error::ZeroSlice)?,
                )?;

                Ok(values)
            }

            fn slice_from_variable(
                variable: &Variable,
                indices: Option<&[usize]>,
                slice_len: Option<&[usize]>,
                values: &mut [Self],
            ) -> error::Result<()> {
                let _indices: Vec<_>;
                let indices = match indices {
                    Some(x) => {
                        if x.len() != variable.dimensions.len() {
                            return Err(error::Error::IndexLen);
                        };
                        x
                    }
                    None => {
                        _indices = variable.dimensions.iter().map(|_| 0).collect();
                        &_indices
                    }
                };
                let _slice_len: Vec<_>;
                let slice_len = match slice_len {
                    Some(x) => {
                        if x.len() != variable.dimensions.len() {
                            return Err(error::Error::SliceLen);
                        }
                        x
                    }
                    None => {
                        _slice_len = variable
                            .dimensions
                            .iter()
                            .zip(indices.iter())
                            .map(|(x, i)| (x.len).wrapping_sub(*i))
                            .collect();
                        &_slice_len
                    }
                };

                let mut values_len = None;
                for i in 0..indices.len() {
                    if indices[i] >= variable.dimensions[i].len {
                        return Err(error::Error::IndexMismatch);
                    }
                    if (indices[i] + slice_len[i]) > variable.dimensions[i].len {
                        return Err(error::Error::SliceMismatch);
                    }
                    values_len = Some(values_len.unwrap_or(1) * slice_len[i]);
                    // Compute the full size of the request values
                    if slice_len[i] == 0 {
                        return Err(error::Error::ZeroSlice);
                    }
                }
                if values_len.is_none() || values_len.unwrap() != values.len() {
                    return Err(error::Error::BufferLen(
                        values.len(),
                        values_len.unwrap_or(0),
                    ));
                }

                let err: nc_type;
                unsafe {
                    let _g = LOCK.lock().unwrap();

                    err = $nc_get_vara_type(
                        variable.ncid,
                        variable.varid,
                        indices.as_ptr(),
                        slice_len.as_ptr(),
                        values.as_mut_ptr(),
                    );
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }
                Ok(())
            }

            // put a SINGLE value into a netCDF variable at the given index
            fn put_value_at(
                variable: &mut Variable,
                indices: Option<&[usize]>,
                value: Self,
            ) -> error::Result<()> {
                // Check the length of `indices`
                let _indices: Vec<usize>;
                let indices = match indices {
                    Some(x) => {
                        if x.len() != variable.dimensions.len() {
                            return Err(error::Error::IndexLen);
                        }
                        for i in 0..x.len() {
                            if x[i] >= variable.dimensions[i].len {
                                return Err(error::Error::IndexMismatch);
                            }
                        }
                        x
                    }
                    None => {
                        _indices = variable.dimensions.iter().map(|_| 0).collect();
                        &_indices
                    }
                };
                let err: nc_type;
                let indices_ptr = indices.as_ptr();
                unsafe {
                    let _g = LOCK.lock().unwrap();
                    err = $nc_put_var1_type(variable.ncid, variable.varid, indices_ptr, &value);
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }

                Ok(())
            }

            // put a SLICE of values into a netCDF variable at the given index
            fn put_values_at(
                variable: &mut Variable,
                indices: Option<&[usize]>,
                slice_len: Option<&[usize]>,
                values: &[Self],
            ) -> error::Result<()> {
                let _indices: Vec<_>;
                let indices = match indices {
                    Some(x) => {
                        if x.len() != variable.dimensions.len() {
                            return Err(error::Error::IndexLen);
                        };
                        x
                    }
                    None => {
                        _indices = variable.dimensions.iter().map(|_| 0).collect();
                        &_indices
                    }
                };

                let _slice_len: Vec<_>;
                let slice_len = match slice_len {
                    Some(x) => x,
                    None => {
                        _slice_len = variable
                            .dimensions
                            .iter()
                            .zip(indices.iter())
                            .map(|(x, i)| (x.len).wrapping_sub(*i))
                            .collect();
                        &_slice_len
                    }
                };

                let mut values_len = None;
                for i in 0..indices.len() {
                    if indices[i] >= variable.dimensions[i].len {
                        return Err(error::Error::IndexMismatch);
                    }
                    if (indices[i] + slice_len[i]) > variable.dimensions[i].len {
                        return Err(error::Error::SliceMismatch);
                    }
                    // Check for empty slice
                    if slice_len[i] == 0 {
                        return Err(error::Error::ZeroSlice);
                    }
                    values_len = Some(values_len.unwrap_or(1) * slice_len[i]);
                }
                if values_len.is_none() || values_len.unwrap() != values.len() {
                    return Err(error::Error::BufferLen(
                        values.len(),
                        values_len.unwrap_or(0),
                    ));
                }

                let err: nc_type;
                unsafe {
                    let _g = LOCK.lock().unwrap();
                    err = $nc_put_vara_type(
                        variable.ncid,
                        variable.varid,
                        indices.as_ptr(),
                        slice_len.as_ptr(),
                        values.as_ptr(),
                    );
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }

                Ok(())
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
        let err;
        unsafe {
            let _l = LOCK.lock().unwrap();
            err = nc_def_var(
                grp_id,
                cname.as_ptr(),
                vartype,
                dimids.len() as _,
                dimids.as_ptr(),
                &mut id,
            );
        }
        if err != NC_NOERR {
            return Err(err.into());
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
        T::single_value_from_variable(self, indices)
    }

    #[cfg(feature = "ndarray")]
    /// Fetches variable
    pub fn values<T: Numeric>(
        &self,
        indices: Option<&[usize]>,
        size_len: Option<&[usize]>,
    ) -> error::Result<ArrayD<T>> {
        T::array_from_variable(self, indices, size_len)
    }

    /// Fetches variable into slice
    /// buffer must be able to hold all the requested elements
    pub fn values_to<T: Numeric>(
        &self,
        buffer: &mut [T],
        indices: Option<&[usize]>,
        size_len: Option<&[usize]>,
    ) -> error::Result<()> {
        T::slice_from_variable(self, indices, size_len, buffer)
    }

    /// Put a single value at `indices`
    pub fn put_value<T: Numeric>(
        &mut self,
        value: T,
        indices: Option<&[usize]>,
    ) -> error::Result<()> {
        T::put_value_at(self, indices, value)
    }

    /// Put a slice of values at `indices`
    pub fn put_values<T: Numeric>(
        &mut self,
        values: &[T],
        indices: Option<&[usize]>,
        slice_len: Option<&[usize]>,
    ) -> error::Result<()> {
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
        let err: nc_type;
        unsafe {
            err = nc_def_var_fill(
                self.ncid,
                self.varid,
                NC_FILL,
                &fill_value as *const T as *const _,
            );
        }
        if err != NC_NOERR {
            return Err(err.into());
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
        let err;
        unsafe {
            err = nc_inq_var_fill(
                self.ncid,
                self.varid,
                &mut nofill,
                &mut location as *mut _ as *mut _,
            );
        }
        if err != NC_NOERR {
            return Err(err.into());
        }
        if nofill == 1 {
            return Ok(None);
        }

        Ok(Some(unsafe { location.assume_init() }))
    }
}

// Write support for all variable types
pub trait PutVar {
    const NCTYPE: nc_type;
    fn put(&self, ncid: nc_type, varid: nc_type) -> error::Result<()>;
}

// This macro implements the trait PutVar for &[$type]
// It just avoid code repetition for all numeric types
// (the only difference between each type beeing the
// netCDF funtion to call and the numeric identifier
// of the type used by the libnetCDF library)
macro_rules! impl_putvar {
    ($type: ty, $nc_type: ident, $nc_put_var: ident) => {
        impl PutVar for &[$type] {
            const NCTYPE: nc_type = $nc_type;
            fn put(&self, ncid: nc_type, varid: nc_type) -> error::Result<()> {
                let err;
                unsafe {
                    let _g = LOCK.lock().unwrap();
                    err = $nc_put_var(ncid, varid, self.as_ptr());
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }
                Ok(())
            }
        }
    };
}
impl_putvar!(u8, NC_UBYTE, nc_put_var_uchar);
impl_putvar!(i8, NC_BYTE, nc_put_var_schar);
impl_putvar!(i16, NC_SHORT, nc_put_var_short);
impl_putvar!(u16, NC_USHORT, nc_put_var_ushort);
impl_putvar!(i32, NC_INT, nc_put_var_int);
impl_putvar!(u32, NC_UINT, nc_put_var_uint);
impl_putvar!(i64, NC_INT64, nc_put_var_longlong);
impl_putvar!(u64, NC_UINT64, nc_put_var_ulonglong);
impl_putvar!(f32, NC_FLOAT, nc_put_var_float);
impl_putvar!(f64, NC_DOUBLE, nc_put_var_double);
