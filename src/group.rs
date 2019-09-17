use super::attribute::{init_attributes, Attribute};
use super::dimension::{init_dimensions, Dimension};
use super::utils::{string_from_c_str, NC_ERRORS};
use super::variable::{init_variable, init_variables, Numeric, Variable};
use super::LOCK;
use netcdf_sys::*;
use std::collections::HashMap;
use std::ffi;
use std::ptr;

#[derive(Debug)]
pub struct Group {
    pub name: String,
    pub id: nc_type,
    pub variables: HashMap<String, Variable>,
    pub attributes: HashMap<String, Attribute>,
    pub dimensions: HashMap<String, Dimension>,
    pub sub_groups: HashMap<String, Group>,
}

// Write support for all variable types
pub trait PutVar {
    fn get_nc_type(&self) -> nc_type;
    fn put(&self, ncid: nc_type, varid: nc_type) -> Result<(), String>;
    fn len(&self) -> usize;
}

// This macro implements the trait PutVar for Vec<$type>
// It just avoid code repetition for all numeric types
// (the only difference between each type beeing the
// netCDF funtion to call and the numeric identifier
// of the type used by the libnetCDF library)
macro_rules! impl_putvar {
    ($type: ty, $nc_type: ident, $nc_put_var: ident) => {
        impl PutVar for Vec<$type> {
            fn get_nc_type(&self) -> nc_type {
                $nc_type
            }
            fn len(&self) -> usize {
                self.len()
            }
            fn put(&self, ncid: nc_type, varid: nc_type) -> Result<(), String> {
                let err;
                unsafe {
                    let _g = LOCK.lock().unwrap();
                    err = $nc_put_var(ncid, varid, self.as_ptr());
                }
                if err != NC_NOERR {
                    return Err(NC_ERRORS.get(&err).unwrap().clone());
                }
                Ok(())
            }
        }
    };
}
impl_putvar!(i8, NC_BYTE, nc_put_var_schar);
impl_putvar!(i16, NC_SHORT, nc_put_var_short);
impl_putvar!(u16, NC_USHORT, nc_put_var_ushort);
impl_putvar!(i32, NC_INT, nc_put_var_int);
impl_putvar!(u32, NC_UINT, nc_put_var_uint);
impl_putvar!(i64, NC_INT64, nc_put_var_longlong);
impl_putvar!(u64, NC_UINT64, nc_put_var_ulonglong);
impl_putvar!(f32, NC_FLOAT, nc_put_var_float);
impl_putvar!(f64, NC_DOUBLE, nc_put_var_double);

// Write support for all attribute types
pub trait PutAttr {
    fn get_nc_type(&self) -> nc_type;
    fn put(&self, ncid: nc_type, varid: nc_type, name: &str) -> Result<(), String>;
}

// This macro implements the trait PutAttr for $type
// It just avoid code repetition for all numeric types
// (the only difference between each type beeing the
// netCDF funtion to call and the numeric identifier
// of the type used by the libnetCDF library)
macro_rules! impl_putattr {
    ($type: ty, $nc_type: ident, $nc_put_att: ident) => {
        impl PutAttr for $type {
            fn get_nc_type(&self) -> nc_type {
                $nc_type
            }
            fn put(&self, ncid: nc_type, varid: nc_type, name: &str) -> Result<(), String> {
                let name_c: ffi::CString = ffi::CString::new(name.clone()).unwrap();
                let err;
                unsafe {
                    let _g = LOCK.lock().unwrap();
                    err = $nc_put_att(ncid, varid, name_c.as_ptr(), $nc_type, 1, self);
                }
                if err != NC_NOERR {
                    return Err(NC_ERRORS.get(&err).unwrap().clone());
                }
                Ok(())
            }
        }
    };
}
impl_putattr!(i8, NC_BYTE, nc_put_att_schar);
impl_putattr!(i16, NC_SHORT, nc_put_att_short);
impl_putattr!(u16, NC_USHORT, nc_put_att_ushort);
impl_putattr!(i32, NC_INT, nc_put_att_int);
impl_putattr!(u32, NC_UINT, nc_put_att_uint);
impl_putattr!(i64, NC_INT64, nc_put_att_longlong);
impl_putattr!(u64, NC_UINT64, nc_put_att_ulonglong);
impl_putattr!(f32, NC_FLOAT, nc_put_att_float);
impl_putattr!(f64, NC_DOUBLE, nc_put_att_double);

impl PutAttr for String {
    fn get_nc_type(&self) -> nc_type {
        NC_CHAR
    }
    fn put(&self, ncid: nc_type, varid: nc_type, name: &str) -> Result<(), String> {
        let name_c: ffi::CString = ffi::CString::new(name.clone()).unwrap();
        let attr_c: ffi::CString = ffi::CString::new(self.clone()).unwrap();
        let err;
        unsafe {
            let _g = LOCK.lock().unwrap();
            err = nc_put_att_text(
                ncid,
                varid,
                name_c.as_ptr(),
                attr_c.to_bytes().len(),
                attr_c.as_ptr(),
            );
        }
        if err != NC_NOERR {
            return Err(NC_ERRORS.get(&err).unwrap().clone());
        }
        Ok(())
    }
}

impl Group {
    pub fn add_attribute<T: PutAttr>(&mut self, name: &str, val: T) -> Result<(), String> {
        val.put(self.id, NC_GLOBAL, name)?;
        self.attributes.insert(
            name.to_string().clone(),
            Attribute {
                name: name.to_string().clone(),
                attrtype: val.get_nc_type(),
                id: 0, // XXX Should Attribute even keep track of an id?
                var_id: NC_GLOBAL,
                file_id: self.id,
            },
        );
        Ok(())
    }

    pub fn add_dimension(&mut self, name: &str, len: usize) -> Result<(), String> {
        let name_c: ffi::CString = ffi::CString::new(name.clone()).unwrap();
        let mut dimid = 0;
        let err;
        unsafe {
            let _g = LOCK.lock().unwrap();
            err = nc_def_dim(self.id, name_c.as_ptr(), len, &mut dimid);
        }
        if err != NC_NOERR {
            return Err(NC_ERRORS.get(&err).unwrap().clone());
        }
        self.dimensions.insert(
            name.to_string().clone(),
            Dimension {
                name: name.to_string().clone(),
                len: len,
                id: dimid,
            },
        );
        Ok(())
    }

    // TODO this should probably take &Vec<&str> instead of &Vec<String>
    pub fn add_variable<T: PutVar>(
        &mut self,
        name: &str,
        dims: &Vec<String>,
        data: &T,
    ) -> Result<(), String> {
        let nctype = data.get_nc_type();
        let grp_id = self.id;
        let var = self.create_variable(name, dims, nctype)?;
        data.put(grp_id, var.id)?;
        Ok(())
    }

    // TODO this should probably take &Vec<&str> instead of &Vec<String>
    pub fn add_variable_with_fill_value<T: PutVar, N: Numeric>(
        &mut self,
        name: &str,
        dims: &Vec<String>,
        data: &T,
        fill_value: N,
    ) -> Result<(), String> {
        let nctype = data.get_nc_type();
        let grp_id = self.id;
        let var = self.create_variable(name, dims, nctype)?;
        var.set_fill_value(fill_value)?;
        data.put(grp_id, var.id)?;
        Ok(())
    }

    // TODO this should probably take &Vec<&str> instead of &Vec<String>
    /// Create a Variable into the dataset, without writting any data into it.
    pub fn create_variable(
        &mut self,
        name: &str,
        dims: &Vec<String>,
        nctype: nc_type,
    ) -> Result<&mut Variable, String> {
        let name_c: ffi::CString = ffi::CString::new(name.clone()).unwrap();
        let mut dimids: Vec<nc_type> = Vec::with_capacity(dims.len());
        let mut var_dims: Vec<Dimension> = Vec::with_capacity(dims.len());
        for dim_name in dims {
            if !self.dimensions.contains_key(dim_name) {
                return Err("Invalid dimension name".to_string());
            }
            var_dims.push(self.dimensions.get(dim_name).unwrap().clone());
        }
        for dim in &var_dims {
            dimids.push(dim.id);
        }
        let mut varid = 0;
        let err;
        unsafe {
            let _g = LOCK.lock().unwrap();
            err = nc_def_var(
                self.id,
                name_c.as_ptr(),
                nctype,
                dims.len() as nc_type,
                dimids.as_ptr(),
                &mut varid,
            );
        }
        if err != NC_NOERR {
            return Err(NC_ERRORS.get(&err).unwrap().clone());
        }
        init_variable(&mut self.variables, self.id, &mut self.dimensions, varid);
        match self.variables.get_mut(name) {
            Some(var) => Ok(var),
            None => Err("Variable creation failed".into()),
        }
    }
}

fn init_sub_groups(
    grp_id: nc_type,
    sub_groups: &mut HashMap<String, Group>,
    parent_dims: &HashMap<String, Dimension>,
) {
    let mut ngrps = 0;
    let mut grpids: Vec<nc_type>;

    // Fetching the group ID's list must be done in 2 steps,
    // 1 - Find out how many groups there are.
    // 2 - Get a list of those group IDs.
    //
    // the function `nc_inq_grps()` fulfill those 2 requests
    // See: http://www.unidata.ucar.edu/software/netcdf/netcdf-4/newdocs/netcdf-c/nc_005finq_005fgrps.html
    unsafe {
        let _g = LOCK.lock().unwrap();
        // Get the number of groups
        let mut err = nc_inq_grps(grp_id, &mut ngrps, ptr::null_mut());
        assert_eq!(err, NC_NOERR);
        // set the group capacity and len to the number of groups
        grpids = Vec::with_capacity(ngrps as usize);
        grpids.set_len(ngrps as usize);
        // Get the list of group IDs
        err = nc_inq_grps(grp_id, &mut ngrps, grpids.as_mut_ptr());
        assert_eq!(err, NC_NOERR);
    }
    for i_grp in 0..ngrps {
        let mut namelen = 0;
        let c_str: &ffi::CStr;
        let str_buf: String;
        unsafe {
            let _g = LOCK.lock().unwrap();
            // name length
            let err = nc_inq_grpname_len(grpids[i_grp as usize], &mut namelen);
            assert_eq!(err, NC_NOERR);
            // name
            let mut buf_vec = vec![0i8; (namelen + 1) as usize];
            let buf_ptr: *mut i8 = buf_vec.as_mut_ptr();
            let err = nc_inq_grpname(grpids[i_grp as usize], buf_ptr);
            assert_eq!(err, NC_NOERR);
            c_str = ffi::CStr::from_ptr(buf_ptr);
            str_buf = string_from_c_str(c_str);
        }

        // Per NetCDF doc, "Dimensions are visible in their groups, and all
        // child groups."
        let mut new_grp = Group {
            name: str_buf.clone(),
            id: grpids[i_grp as usize],
            variables: HashMap::new(),
            attributes: HashMap::new(),
            dimensions: parent_dims.clone(),
            sub_groups: HashMap::new(),
        };
        init_group(&mut new_grp);
        sub_groups.insert(str_buf.clone(), new_grp);
    }
}

pub fn init_group(grp: &mut Group) {
    init_dimensions(&mut grp.dimensions, grp.id);
    init_attributes(&mut grp.attributes, grp.id, NC_GLOBAL, -1);
    init_variables(&mut grp.variables, grp.id, &grp.dimensions);
    init_sub_groups(grp.id, &mut grp.sub_groups, &grp.dimensions);
}
