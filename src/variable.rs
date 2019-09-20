use super::attribute::AnyValue;
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
                            return Err(
                                "`indices` must has the same length as the variable dimensions"
                                    .into(),
                            );
                        }
                        for i in 0..x.len() {
                            if x[i] >= variable.dimensions[i].len {
                                return Err(
                                    "requested index is bigger than the dimension length".into()
                                );
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
                    values
                        .as_slice_mut()
                        .ok_or_else(|| error::Error::Crate("Values is emtpy".to_string()))?,
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
                            return Err(
                                "`indices` must has the same length as the variable dimensions"
                                    .into(),
                            );
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
                            return Err(
                                "`slice_len` must have the same length as the variable dimsensions"
                                    .into(),
                            );
                        }
                        x
                    }
                    None => {
                        _slice_len = variable.dimensions.iter().map(|x| x.len).collect();
                        &_slice_len
                    }
                };

                for i in 0..indices.len() {
                    if indices[i] >= variable.dimensions[i].len {
                        return Err("requested index is bigger than the dimension length".into());
                    }
                    if (indices[i] + slice_len[i]) > variable.dimensions[i].len {
                        return Err("requested slice is bigger than the dimension length".into());
                    }
                    // Compute the full size of the request values
                    if slice_len[i] == 0 {
                        return Err("Each slice element must be superior than 0".into());
                    }
                }

                if slice_len.iter().fold(1, |acc, x| acc * x) > values.len() {
                    return Err("Number of elements exceeds space in buffer".into());
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
                            return Err(
                                "`indices` must has the same length as the variable dimensions"
                                    .into(),
                            );
                        }
                        for i in 0..x.len() {
                            if x[i] >= variable.dimensions[i].len {
                                return Err(
                                    "requested index is bigger than the dimension length".into()
                                );
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
                println!("{:?}", values);
                let _indices: Vec<_>;
                let indices = match indices {
                    Some(x) => {
                        if x.len() != variable.dimensions.len() {
                            return Err(
                                "`slice` must has the same length as the variable dimensions"
                                    .into(),
                            );
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
                        _slice_len = variable.dimensions.iter().map(|x| x.len).collect();
                        &_slice_len
                    }
                };

                let mut values_len = 0;
                for i in 0..indices.len() {
                    if indices[i] >= variable.dimensions[i].len {
                        return Err("requested index is bigger than the dimension length".into());
                    }
                    if (indices[i] + slice_len[i]) > variable.dimensions[i].len {
                        return Err("requested slice is bigger than the dimension length".into());
                    }
                    // Check for empty slice
                    if slice_len[i] == 0 {
                        return Err("Each slice element must be superior than 0".into());
                    }
                    values_len += slice_len[i];
                }
                if values_len != values.len() {
                    return Err("number of element in `values` doesn't match `slice_len`".into());
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
        T: Into<AnyValue>,
    {
        let att = Attribute::put(self.ncid, self.varid, name, val.into())?;
        self.attributes.insert(name.to_string().clone(), att);
        Ok(())
    }

    ///  Fetches one specific value at specific indices
    ///  indices must has the same length as self.dimensions.
    pub fn get_value<T: Numeric>(&self, indices: Option<&[usize]>) -> error::Result<T> {
        T::single_value_from_variable(self, indices)
    }

    #[cfg(feature = "ndarray")]
    /// Fetches variable
    pub fn get_values<'a, T: Numeric>(
        &self,
        indices: Option<&[usize]>,
        size_len: Option<&[usize]>,
    ) -> error::Result<ArrayD<T>> {
        T::array_from_variable(self, indices, size_len)
    }

    /// Fetches variable into slice
    /// buffer must be able to hold all the requested elements
    pub fn get_values_to<'a, T: Numeric>(
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
    pub fn put_values<'a, T: Numeric>(
        &mut self,
        values: &[T],
        indices: Option<&[usize]>,
        slice_len: Option<&[usize]>,
    ) -> error::Result<()> {
        T::put_values_at(self, indices, slice_len, values)
    }

    /// Set a Fill Value
    pub fn set_fill_value<T: Numeric>(&mut self, fill_value: T) -> error::Result<()> {
        let err: nc_type;
        unsafe {
            let _g = LOCK.lock().unwrap();
            err = nc_def_var_fill(
                self.ncid,
                self.varid,
                0,
                &fill_value as *const T as *const _,
            );
        }
        if err != NC_NOERR {
            return Err(err.into());
        }
        Ok(())
    }
}
