use super::attribute::{init_attributes, Attribute};
use super::dimension::Dimension;
use super::group::PutAttr;
use super::utils::{string_from_c_str, NC_ERRORS};
use super::LOCK;
use ndarray::ArrayD;
use netcdf_sys::*;
use std::collections::HashMap;
use std::ffi;
use std::marker::Sized;

macro_rules! get_var_as_type {
    ( $me:ident, $nc_type:ident, $vec_type:ty, $nc_fn:ident , $cast:ident ) => {{
        if (!$cast) && ($me.vartype != $nc_type) {
            return Err("Types are not equivalent and cast==false".to_string());
        }
        let mut buf: Vec<$vec_type> = Vec::with_capacity($me.len as usize);
        let err: nc_type;
        unsafe {
            let _g = LOCK.lock().unwrap();
            buf.set_len($me.len as usize);
            err = $nc_fn($me.grp_id, $me.id, buf.as_mut_ptr());
        }
        if err != NC_NOERR {
            return Err(NC_ERRORS.get(&err).unwrap().clone());
        }
        Ok(buf)
    }};
}

/// This trait allow an implicit cast when fetching
/// a netCDF variable
pub trait Numeric
where
    Self: Sized,
{
    /// Returns a single indexed value of the variable as Self
    fn single_value_from_variable(variable: &Variable, indices: &[usize]) -> Result<Self, String>;
    /// Returns an ndarray of the variable
    fn array_from_variable(
        variable: &Variable,
        indices: Option<&[usize]>,
        slice_len: Option<&[usize]>,
    ) -> Result<ArrayD<Self>, String>;
    /// Returns a slice of the variable as Vec<Self>
    fn slice_from_variable(
        variable: &Variable,
        indices: Option<&[usize]>,
        slice_len: Option<&[usize]>,
        values: &mut [Self],
    ) -> Result<(), String>;
    /// Put a single value into a netCDF variable
    fn put_value_at(variable: &mut Variable, indices: &[usize], value: Self) -> Result<(), String>;
    /// put a SLICE of values into a netCDF variable at the given index
    fn put_values_at(
        variable: &mut Variable,
        indices: Option<&[usize]>,
        slice_len: Option<&[usize]>,
        values: &[Self],
    ) -> Result<(), String>;
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
            // fetch ONE value from variable using `$nc_get_var1`
            fn single_value_from_variable(
                variable: &Variable,
                indices: &[usize],
            ) -> Result<$sized_type, String> {
                // Check the length of `indices`
                if indices.len() != variable.dimensions.len() {
                    return Err(
                        "`indices` must has the same length as the variable dimensions".into(),
                    );
                }
                for i in 0..indices.len() {
                    if indices[i] >= variable.dimensions[i].len {
                        return Err("requested index is bigger than the dimension length".into());
                    }
                }
                // initialize `buff` to 0
                let mut buff: $sized_type = 0 as $sized_type;
                let err: nc_type;
                // Get a pointer to an array
                let indices_ptr = indices.as_ptr();
                unsafe {
                    let _g = LOCK.lock().unwrap();
                    err = $nc_get_var1_type(variable.grp_id, variable.id, indices_ptr, &mut buff);
                }
                if err != NC_NOERR {
                    return Err(NC_ERRORS.get(&err).unwrap().clone());
                }
                Ok(buff)
            }

            fn array_from_variable(
                variable: &Variable,
                indices: Option<&[usize]>,
                slice_len: Option<&[usize]>,
            ) -> Result<ArrayD<$sized_type>, String> {
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
                        .ok_or_else(|| "Values is emtpy".to_string())?,
                )?;

                Ok(values)
            }

            fn slice_from_variable(
                variable: &Variable,
                indices: Option<&[usize]>,
                slice_len: Option<&[usize]>,
                values: &mut [Self],
            ) -> Result<(), String> {
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
                        variable.grp_id,
                        variable.id,
                        indices.as_ptr(),
                        slice_len.as_ptr(),
                        values.as_mut_ptr(),
                    );
                }
                if err != NC_NOERR {
                    return Err(NC_ERRORS.get(&err).unwrap().clone());
                }
                Ok(())
            }

            // put a SINGLE value into a netCDF variable at the given index
            fn put_value_at(
                variable: &mut Variable,
                indices: &[usize],
                value: Self,
            ) -> Result<(), String> {
                // Check the length of `indices`
                if indices.len() != variable.dimensions.len() {
                    return Err(
                        "`indices` must has the same length as the variable dimensions".into(),
                    );
                }
                for i in 0..indices.len() {
                    if indices[i] >= variable.dimensions[i].len {
                        return Err("requested index is bigger than the dimension length".into());
                    }
                }
                let err: nc_type;
                let indices_ptr = indices.as_ptr();
                unsafe {
                    let _g = LOCK.lock().unwrap();
                    err = $nc_put_var1_type(variable.grp_id, variable.id, indices_ptr, &value);
                }
                if err != NC_NOERR {
                    return Err(NC_ERRORS.get(&err).unwrap().clone());
                }

                Ok(())
            }

            // put a SLICE of values into a netCDF variable at the given index
            fn put_values_at(
                variable: &mut Variable,
                indices: Option<&[usize]>,
                slice_len: Option<&[usize]>,
                values: &[Self],
            ) -> Result<(), String> {
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
                        variable.grp_id,
                        variable.id,
                        indices.as_ptr(),
                        slice_len.as_ptr(),
                        values.as_ptr(),
                    );
                }
                if err != NC_NOERR {
                    return Err(NC_ERRORS.get(&err).unwrap().clone());
                }

                Ok(())
            }
        }
    };
}
impl_numeric!(
    u8,
    NC_CHAR,
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

/// This struct defines a netCDF variable.
#[derive(Debug)]
pub struct Variable {
    /// The variable name
    pub name: String,
    pub attributes: HashMap<String, Attribute>,
    pub dimensions: Vec<Dimension>,
    /// the netcdf variable type identifier (from netcdf-sys)
    pub vartype: nc_type,
    pub id: nc_type,
    /// total length; the product of all dim lengths
    pub len: usize,
    pub grp_id: nc_type,
}

impl Variable {
    pub fn get_char(&self, cast: bool) -> Result<Vec<u8>, String> {
        get_var_as_type!(self, NC_CHAR, u8, nc_get_var_uchar, cast)
    }
    pub fn get_byte(&self, cast: bool) -> Result<Vec<i8>, String> {
        get_var_as_type!(self, NC_BYTE, i8, nc_get_var_schar, cast)
    }
    pub fn get_short(&self, cast: bool) -> Result<Vec<i16>, String> {
        get_var_as_type!(self, NC_SHORT, i16, nc_get_var_short, cast)
    }
    pub fn get_ushort(&self, cast: bool) -> Result<Vec<u16>, String> {
        get_var_as_type!(self, NC_USHORT, u16, nc_get_var_ushort, cast)
    }
    pub fn get_int(&self, cast: bool) -> Result<Vec<i32>, String> {
        get_var_as_type!(self, NC_INT, i32, nc_get_var_int, cast)
    }
    pub fn get_uint(&self, cast: bool) -> Result<Vec<u32>, String> {
        get_var_as_type!(self, NC_UINT, u32, nc_get_var_uint, cast)
    }
    pub fn get_int64(&self, cast: bool) -> Result<Vec<i64>, String> {
        get_var_as_type!(self, NC_INT64, i64, nc_get_var_longlong, cast)
    }
    pub fn get_uint64(&self, cast: bool) -> Result<Vec<u64>, String> {
        get_var_as_type!(self, NC_UINT64, u64, nc_get_var_ulonglong, cast)
    }
    pub fn get_float(&self, cast: bool) -> Result<Vec<f32>, String> {
        get_var_as_type!(self, NC_FLOAT, f32, nc_get_var_float, cast)
    }
    pub fn get_double(&self, cast: bool) -> Result<Vec<f64>, String> {
        get_var_as_type!(self, NC_DOUBLE, f64, nc_get_var_double, cast)
    }

    pub fn add_attribute<T: PutAttr>(&mut self, name: &str, val: T) -> Result<(), String> {
        val.put(self.grp_id, self.id, name)?;
        self.attributes.insert(
            name.to_string().clone(),
            Attribute {
                name: name.to_string().clone(),
                attrtype: val.get_nc_type(),
                id: 0, // XXX Should Attribute even keep track of an id?
                var_id: self.id,
                file_id: self.grp_id,
            },
        );
        Ok(())
    }

    ///  Fetches one specific value at specific indices
    ///  indices must has the same length as self.dimensions.
    pub fn value_at<T: Numeric>(&self, indices: &[usize]) -> Result<T, String> {
        T::single_value_from_variable(self, indices)
    }

    /// Fetches variable
    pub fn values<'a, T: Numeric, O, P>(&self, indices: O, size_len: P) -> Result<ArrayD<T>, String>
    where
        O: Into<Option<&'a [usize]>>,
        P: Into<Option<&'a [usize]>>,
    {
        T::array_from_variable(self, indices.into(), size_len.into())
    }

    /// Fetches variable into slice
    /// buffer must be able to hold all the requested elements
    pub fn values_to<'a, T: Numeric, O, P>(
        &self,
        indices: O,
        size_len: P,
        buffer: &mut [T],
    ) -> Result<(), String>
    where
        O: Into<Option<&'a [usize]>>,
        P: Into<Option<&'a [usize]>>,
    {
        T::slice_from_variable(self, indices.into(), size_len.into(), buffer)
    }

    /// Put a single value at `indices`
    pub fn put_value_at<T: Numeric>(&mut self, value: T, indices: &[usize]) -> Result<(), String> {
        T::put_value_at(self, indices, value)
    }

    /// Put a slice of values at `indices`
    pub fn put_values_at<'a, T: Numeric, O, P>(
        &mut self,
        values: &[T],
        indices: O,
        slice_len: P,
    ) -> Result<(), String>
    where
        O: Into<Option<&'a [usize]>>,
        P: Into<Option<&'a [usize]>>,
    {
        T::put_values_at(self, indices.into(), slice_len.into(), values)
    }

    /// Set a Fill Value
    pub fn set_fill_value<T: Numeric>(&mut self, fill_value: T) -> Result<(), String> {
        let err: nc_type;
        unsafe {
            let _g = LOCK.lock().unwrap();
            err = nc_def_var_fill(self.grp_id, self.id, 0, &fill_value as *const T as *const _);
        }
        if err != NC_NOERR {
            return Err(NC_ERRORS.get(&err).unwrap().clone());
        }
        self.update_attributes()?;
        Ok(())
    }

    /// update self.attributes, (sync cached attribute and the file)
    fn update_attributes(&mut self) -> Result<(), String> {
        let mut natts: nc_type = 0;
        let err: nc_type;
        unsafe {
            let _g = LOCK.lock().unwrap();
            err = nc_inq_varnatts(self.grp_id, self.id, &mut natts);
        }
        if err != NC_NOERR {
            return Err(NC_ERRORS.get(&err).unwrap().clone());
        }
        let (grp_id, var_id) = (self.grp_id, self.id);
        self.attributes.clear();
        init_attributes(&mut self.attributes, grp_id, var_id, natts);
        Ok(())
    }
}

pub fn init_variables(
    vars: &mut HashMap<String, Variable>,
    grp_id: nc_type,
    grp_dims: &HashMap<String, Dimension>,
) {
    // determine number of vars
    let mut nvars = 0;
    unsafe {
        let _g = LOCK.lock().unwrap();
        let err = nc_inq_nvars(grp_id, &mut nvars);
        assert_eq!(err, NC_NOERR);
    }
    for i_var in 0..nvars {
        init_variable(vars, grp_id, grp_dims, i_var);
    }
}

/// Creates and add a `Variable` Objects, from the dataset
pub fn init_variable(
    vars: &mut HashMap<String, Variable>,
    grp_id: nc_type,
    grp_dims: &HashMap<String, Dimension>,
    varid: nc_type,
) {
    // read each dim name and length
    let mut buf_vec = vec![0i8; (NC_MAX_NAME + 1) as usize];
    let c_str: &ffi::CStr;
    let mut var_type = 0;
    let mut ndims = 0;
    let mut dimids: Vec<_> = Vec::with_capacity(NC_MAX_DIMS as usize);
    let mut natts: nc_type = 0;
    unsafe {
        let _g = LOCK.lock().unwrap();
        let buf_ptr: *mut i8 = buf_vec.as_mut_ptr();
        let err = nc_inq_var(
            grp_id,
            varid,
            buf_ptr,
            &mut var_type,
            &mut ndims,
            dimids.as_mut_ptr(),
            &mut natts,
        );
        dimids.set_len(ndims as usize);
        assert_eq!(err, NC_NOERR);
        c_str = ffi::CStr::from_ptr(buf_ptr);
    }
    let str_buf: String = string_from_c_str(c_str);
    let mut attr_map: HashMap<String, Attribute> = HashMap::new();
    init_attributes(&mut attr_map, grp_id, varid, natts);
    // var dims should always be a subset of the group dims:
    let mut dim_vec: Vec<Dimension> = Vec::new();
    let mut len = 1;
    for dimid in dimids {
        // maintaining dim order is crucial here so we can maintain
        // rule that "last dim varies fastest" in our 1D return Vec
        for (_, grp_dim) in grp_dims {
            if dimid == grp_dim.id {
                len *= grp_dim.len;
                dim_vec.push(grp_dim.clone());
                break;
            }
        }
    }
    vars.insert(
        str_buf.clone(),
        Variable {
            name: str_buf.clone(),
            attributes: attr_map,
            dimensions: dim_vec,
            vartype: var_type,
            len: len,
            id: varid,
            grp_id: grp_id,
        },
    );
}
