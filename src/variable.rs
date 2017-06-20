use std::marker::Sized;
use std::ffi;
use std::collections::HashMap;
use netcdf_sys::*;
use dimension::Dimension;
use group::PutAttr;
use attribute::{init_attributes, Attribute};
use string_from_c_str;
use NC_ERRORS;
use std::error::Error;
use ndarray::{Array1,ArrayD,IxDyn};
use libc;

macro_rules! get_var_as_type {
    ( $me:ident, $nc_type:ident, $vec_type:ty, $nc_fn:ident , $cast:ident ) 
        => 
    {{
        if (!$cast) && ($me.vartype != $nc_type) {
            return Err("Types are not equivalent and cast==false".to_string());
        }
        let mut buf: Vec<$vec_type> = Vec::with_capacity($me.len as usize);
        let err: i32;
        unsafe {
            let _g = libnetcdf_lock.lock().unwrap();
            buf.set_len($me.len as usize);
            err = $nc_fn($me.file_id, $me.id, buf.as_mut_ptr());
        }
        if err != nc_noerr {
            return Err(NC_ERRORS.get(&err).unwrap().clone());
        }
        Ok(buf)
    }};
}

/// This trait allow an implicit cast when fetching 
/// a netCDF variable
pub trait Numeric {

    fn from_variable(variable: &Variable) -> Result<Vec<Self>, String>
        where Self: Sized;
    fn zeros() -> Self
        where Self: Sized;
}
// This macro implements the trait Numeric for the type "sized_type"
// if "sized_type" is equivalent to "nc_type" (the constant from the libnetcdf)
// the function "nc_fn" will be called with caste set to false, set to true otherwise.
// "nc_fn" should had been generated using "get_var_as_type"
macro_rules! impl_getter {
    ($sized_type: ty, $nc_type: ident, $nc_fn: ident) => {
        impl Numeric for $sized_type {
            fn from_variable(variable: &Variable) -> Result<Vec<$sized_type>, String> {
                let cast = variable.vartype != $nc_type;
                get_var_as_type!(variable, $nc_type, $sized_type, $nc_fn, cast)
            }
            fn zeros() -> $sized_type {
                0 as $sized_type
            }
        }
    }
}
impl_getter!(u8, nc_char, nc_get_var_uchar);
impl_getter!(i8, nc_byte, nc_get_var_schar);
impl_getter!(i16, nc_short, nc_get_var_short);
impl_getter!(u16, nc_ushort, nc_get_var_ushort);
impl_getter!(i32, nc_int, nc_get_var_int);
impl_getter!(u32, nc_uint, nc_get_var_uint);
impl_getter!(i64, nc_int64, nc_get_var_longlong);
impl_getter!(u64, nc_uint64, nc_get_var_ulonglong);
impl_getter!(f32, nc_float, nc_get_var_float);
impl_getter!(f64, nc_double, nc_get_var_double);

/// This struct defines a netCDF variable.
pub struct Variable {
    /// The variable name
    pub name : String,
    pub attributes : HashMap<String, Attribute>,
    pub dimensions : Vec<Dimension>,
    /// the netcdf variable type identifier (from netcdf-sys)
    pub vartype : i32,
    pub id: i32,
    /// total length; the product of all dim lengths
    pub len: u64, 
    pub file_id: i32,
}

impl Variable {
    pub fn get_char(&self, cast: bool) -> Result<Vec<u8>, String> {
        get_var_as_type!(self, nc_char, u8, nc_get_var_uchar, cast)
    }
    pub fn get_byte(&self, cast: bool) -> Result<Vec<i8>, String> {
        get_var_as_type!(self, nc_byte, i8, nc_get_var_schar, cast)
    }
    pub fn get_short(&self, cast: bool) -> Result<Vec<i16>, String> {
        get_var_as_type!(self, nc_short, i16, nc_get_var_short, cast)
    }
    pub fn get_ushort(&self, cast: bool) -> Result<Vec<u16>, String> {
        get_var_as_type!(self, nc_ushort, u16, nc_get_var_ushort, cast)
    }
    pub fn get_int(&self, cast: bool) -> Result<Vec<i32>, String> {
        get_var_as_type!(self, nc_int, i32, nc_get_var_int, cast)
    }
    pub fn get_uint(&self, cast: bool) -> Result<Vec<u32>, String> {
        get_var_as_type!(self, nc_uint, u32, nc_get_var_uint, cast)
    }
    pub fn get_int64(&self, cast: bool) -> Result<Vec<i64>, String> {
        get_var_as_type!(self, nc_int64, i64, nc_get_var_longlong, cast)
    }
    pub fn get_uint64(&self, cast: bool) -> Result<Vec<u64>, String> {
        get_var_as_type!(self, nc_uint64, u64, nc_get_var_ulonglong, cast)
    }
    pub fn get_float(&self, cast: bool) -> Result<Vec<f32>, String> {
        get_var_as_type!(self, nc_float, f32, nc_get_var_float, cast)
    }
    pub fn get_double(&self, cast: bool) -> Result<Vec<f64>, String> {
        get_var_as_type!(self, nc_double, f64, nc_get_var_double, cast)
    }

    pub fn add_attribute<T: PutAttr>(&mut self, name: &str, val: T) 
            -> Result<(), String> {
        try!(val.put(self.file_id, self.id, name));
        self.attributes.insert(
                name.to_string().clone(),
                Attribute {
                    name: name.to_string().clone(),
                    attrtype: val.get_nc_type(),
                    id: 0, // XXX Should Attribute even keep track of an id?
                    var_id: self.id,
                    file_id: self.file_id
                }
            );
        Ok(())
    }

    /// Fetchs variable values.
    ///
    /// ```
    /// // Each values will be implicitly casted to a f64 if needed
    /// // let values: Vec<f64> = some_variable.values().unwrap();
    /// ```
    ///
    pub fn values<T: Numeric>(&self) -> Result<Vec<T>, String> {
        T::from_variable(self)
    }
    
    /// Fetchs one specific value at specific indices
    ///  indices must has the same length as self.dimensions.
    pub fn value_at<T: Numeric>(&self, indices: &[usize]) -> Result<T, String> {
        // Check the length of `indices`
        if indices.len() != self.dimensions.len() {
            return Err("`indices` must has the same length as the variable dimensions".into());
        }
        for i in 0..indices.len() {
            if (indices[i] as u64) >= self.dimensions[i].len {
                return Err("requested index is bigger than the dimension length".into());
            }
        }

        // initialize `buff` to 0
        let mut buff: T = T::zeros();
        // create a c_void pointer from `buff`
        let buff_ptr = &mut buff as *mut _ as *mut libc::c_void;
        let err: i32;

        // Get a pointer to an array [size_t]
        let indices: Vec<size_t> = indices.iter().map(|i| *i as size_t).collect();
        let indices_ptr = indices.as_slice().as_ptr();
        unsafe {
            let _g = libnetcdf_lock.lock().unwrap();
            //fn nc_get_var1(ncid: libc::c_int, varid: libc::c_int, indexp: *const size_t, ip: *mut libc::c_void)
            err = nc_get_var1(self.file_id, self.id, indices_ptr, buff_ptr);
        }
        if err != nc_noerr {
            return Err(NC_ERRORS.get(&err).unwrap().clone());
        }
        Ok(buff)
    }

    /// Fetchs a slice of values
    ///  indices must has the same length as self.dimensions.
    pub fn values_at<T: Numeric>(&self, indices: &[usize], slice_len: &[usize]) -> Result<Vec<T>, String> {
        // Check the length of `indices`
        if indices.len() != self.dimensions.len() {
            return Err("`indices` must has the same length as the variable dimensions".into());
        }
        if indices.len() != slice_len.len() {
            return Err("`slice` must has the same length as the variable dimensions".into());
        }
        let mut values: Vec<T>;
        let mut values_len: usize = 1;
        for i in 0..indices.len() {
            if (indices[i] as u64) >= self.dimensions[i].len {
                return Err("requested index is bigger than the dimension length".into());
            }
            if ((indices[i] + slice_len[i]) as u64) > self.dimensions[i].len {
                return Err("requested slice is bigger than the dimension length".into());
            }
            // Compute the full size of the request values
            if slice_len[i] > 0 {
                values_len *= slice_len[i];
            }
        }

        let err: i32;

        // Get a pointer to an array [size_t]
        let indices: Vec<size_t> = indices.iter().map(|i| *i as size_t).collect();
        let slice: Vec<size_t> = slice_len.iter().map(|i| *i as size_t).collect();
        unsafe {
            let _g = libnetcdf_lock.lock().unwrap();

            values = Vec::with_capacity(values_len);
            values.set_len(values_len);
            let buff_ptr = values.as_mut_ptr() as *mut _ as *mut libc::c_void;

            err = nc_get_vara(
                self.file_id,
                self.id,
                indices.as_slice().as_ptr(),
                slice.as_slice().as_ptr(),
                buff_ptr
            );
        }
        if err != nc_noerr {
            return Err(NC_ERRORS.get(&err).unwrap().clone());
        }
        Ok(values)
    }

    /// Fetchs variable values as a ndarray.
    ///
    /// ```
    /// // Each values will be implicitly casted to a f64 if needed
    /// // let values: ArrayD<f64> = some_variable.as_array().unwrap();
    /// ```
    ///
    pub fn as_array<T: Numeric>(&self) -> Result<ArrayD<T>, Box<Error>> {
        let mut dims: Vec<usize> = Vec::new();
        for dim in &self.dimensions {
            dims.push(dim.len as usize);
        }
        let values = self.values()?;
        let array = Array1::<T>::from_vec(values);
        Ok(array.into_shape(dims)?)
    }
}

pub fn init_variables(vars: &mut HashMap<String, Variable>, grp_id: i32,
                  grp_dims: &HashMap<String, Dimension>) {
    // determine number of vars
    let mut nvars = 0i32;
    unsafe {
        let _g = libnetcdf_lock.lock().unwrap();
        let err = nc_inq_nvars(grp_id, &mut nvars);
        assert_eq!(err, nc_noerr);
    }
    // read each dim name and length
    for i_var in 0..nvars {
        let mut buf_vec = vec![0i8; (nc_max_name + 1) as usize];
        let c_str: &ffi::CStr;
        let mut var_type : i32 = 0;
        let mut ndims : i32 = 0;
        let mut dimids : Vec<i32> = Vec::with_capacity(nc_max_dims as usize);
        let mut natts : i32 = 0;
        unsafe {
            let _g = libnetcdf_lock.lock().unwrap();
            let buf_ptr : *mut i8 = buf_vec.as_mut_ptr();
            let err = nc_inq_var(grp_id, i_var, buf_ptr,
                                    &mut var_type, &mut ndims,
                                    dimids.as_mut_ptr(), &mut natts);
            dimids.set_len(ndims as usize);
            assert_eq!(err, nc_noerr);
            c_str = ffi::CStr::from_ptr(buf_ptr);
        }
        let str_buf: String = string_from_c_str(c_str);
        let mut attr_map : HashMap<String, Attribute> = HashMap::new();
        init_attributes(&mut attr_map, grp_id, i_var, natts);
        // var dims should always be a subset of the group dims:
        let mut dim_vec : Vec<Dimension> = Vec::new();
        let mut len : u64 = 1;
        for dimid in dimids {
            // maintaining dim order is crucial here so we can maintain
            // rule that "last dim varies fastest" in our 1D return Vec
            for (_, grp_dim) in grp_dims {
                if dimid == grp_dim.id {
                    len *= grp_dim.len;
                    dim_vec.push(grp_dim.clone());
                    break
                }
            }
        }
        vars.insert(str_buf.clone(),
                      Variable{name: str_buf.clone(),
                          attributes: attr_map,
                          dimensions: dim_vec,
                          vartype: var_type,
                          len: len,
                          id: i_var,
                          file_id: grp_id});
    }
}

