use std::collections::HashMap;
use std::ffi;
use netcdf_sys::*;
use dimension::{init_dimensions, Dimension};
use attribute::{init_attributes, Attribute};
use variable::{init_variables, Variable};
use string_from_c_str;
use NC_ERRORS;

pub struct Group {
    pub name : String,
    pub id : i32,
    pub variables : HashMap<String, Variable>,
    pub attributes : HashMap<String, Attribute>,
    pub dimensions : HashMap<String, Dimension>,
    pub sub_groups : HashMap<String, Group>,
}

macro_rules! put_var_as_type {
    ( $me:ident, $ncid:ident, $varid:ident, $nc_fn:ident )
        => 
    {{
        let err : i32;
        unsafe {
            let _g = libnetcdf_lock.lock().unwrap();
            err = $nc_fn($ncid, $varid, $me.as_ptr());
        }
        if err != nc_noerr {
            return Err(NC_ERRORS.get(&err).unwrap().clone());
        }
        Ok(())
    }};
}

macro_rules! put_attr_as_type {
    ( $me:ident, $attname:ident, $ncid:ident, $nctype: ident, 
      $varid:ident, $nc_fn:ident )
        => 
    {{
        let name_c: ffi::CString = ffi::CString::new($attname.clone()).unwrap();
        let err : i32;
        unsafe {
            let _g = libnetcdf_lock.lock().unwrap();
            err = $nc_fn($ncid, $varid, name_c.as_ptr(), $nctype, 1, $me);
        }
        if err != nc_noerr {
            return Err(NC_ERRORS.get(&err).unwrap().clone());
        }
        Ok(())
    }};
}

// Write support for all variable types ... excuse the repetition :(
pub trait PutVar {
    fn get_nc_type(&self) -> i32;
    fn put(&self, ncid: i32, varid: i32) -> Result<(), String> ;
    fn len(&self) -> usize;
}

impl PutVar for Vec<i8> {
    fn get_nc_type(&self) -> i32 { nc_byte }
    fn len(&self) -> usize { self.len() }
    fn put(&self, ncid: i32, varid: i32) -> Result<(), String> {
        put_var_as_type!(self, ncid, varid, nc_put_var_schar)
    }
}

impl PutVar for Vec<i16> {
    fn get_nc_type(&self) -> i32 { nc_short }
    fn len(&self) -> usize { self.len() }
    fn put(&self, ncid: i32, varid: i32) -> Result<(), String> {
        put_var_as_type!(self, ncid, varid, nc_put_var_short)
    }
}

impl PutVar for Vec<u16> {
    fn get_nc_type(&self) -> i32 { nc_ushort }
    fn len(&self) -> usize { self.len() }
    fn put(&self, ncid: i32, varid: i32) -> Result<(), String> {
        put_var_as_type!(self, ncid, varid, nc_put_var_ushort)
    }
}

impl PutVar for Vec<i32> {
    fn get_nc_type(&self) -> i32 { nc_int }
    fn len(&self) -> usize { self.len() }
    fn put(&self, ncid: i32, varid: i32) -> Result<(), String> {
        put_var_as_type!(self, ncid, varid, nc_put_var_int)
    }
}

impl PutVar for Vec<u32> {
    fn get_nc_type(&self) -> i32 { nc_uint }
    fn len(&self) -> usize { self.len() }
    fn put(&self, ncid: i32, varid: i32) -> Result<(), String> {
        put_var_as_type!(self, ncid, varid, nc_put_var_uint)
    }
}

impl PutVar for Vec<i64> {
    fn get_nc_type(&self) -> i32 { nc_int64 }
    fn len(&self) -> usize { self.len() }
    fn put(&self, ncid: i32, varid: i32) -> Result<(), String> {
        put_var_as_type!(self, ncid, varid, nc_put_var_longlong)
    }
}

impl PutVar for Vec<u64> {
    fn get_nc_type(&self) -> i32 { nc_uint64 }
    fn len(&self) -> usize { self.len() }
    fn put(&self, ncid: i32, varid: i32) -> Result<(), String> {
        put_var_as_type!(self, ncid, varid, nc_put_var_ulonglong)
    }
}

impl PutVar for Vec<f32> {
    fn get_nc_type(&self) -> i32 { nc_float }
    fn len(&self) -> usize { self.len() }
    fn put(&self, ncid: i32, varid: i32) -> Result<(), String> {
        put_var_as_type!(self, ncid, varid, nc_put_var_float)
    }
}

impl PutVar for Vec<f64> {
    fn get_nc_type(&self) -> i32 { nc_double }
    fn len(&self) -> usize { self.len() }
    fn put(&self, ncid: i32, varid: i32) -> Result<(), String> {
        put_var_as_type!(self, ncid, varid, nc_put_var_double)
    }
}


// Write support for all attribute types
pub trait PutAttr {
    fn get_nc_type(&self) -> i32;
    fn put(&self, ncid: i32, varid: i32, name: &str) -> Result<(), String> ;
}

impl PutAttr for i8 {
    fn get_nc_type(&self) -> i32 { nc_byte }
    fn put(&self, ncid: i32, varid: i32, name: &str) -> Result<(), String> {
        put_attr_as_type!(self, name, ncid, nc_byte, varid, nc_put_att_schar)
    }
}

impl PutAttr for i16 {
    fn get_nc_type(&self) -> i32 { nc_short }
    fn put(&self, ncid: i32, varid: i32, name: &str) -> Result<(), String> {
        put_attr_as_type!(self, name, ncid, nc_short, varid, nc_put_att_short)
    }
}

impl PutAttr for u16 {
    fn get_nc_type(&self) -> i32 { nc_ushort }
    fn put(&self, ncid: i32, varid: i32, name: &str) -> Result<(), String> {
        put_attr_as_type!(self, name, ncid, nc_ushort, varid, nc_put_att_ushort)
    }
}

impl PutAttr for i32 {
    fn get_nc_type(&self) -> i32 { nc_int }
    fn put(&self, ncid: i32, varid: i32, name: &str) -> Result<(), String> {
        put_attr_as_type!(self, name, ncid, nc_int, varid, nc_put_att_int)
    }
}

impl PutAttr for u32 {
    fn get_nc_type(&self) -> i32 { nc_uint }
    fn put(&self, ncid: i32, varid: i32, name: &str) -> Result<(), String> {
        put_attr_as_type!(self, name, ncid, nc_uint, varid, nc_put_att_uint)
    }
}

impl PutAttr for i64 {
    fn get_nc_type(&self) -> i32 { nc_int64 }
    fn put(&self, ncid: i32, varid: i32, name: &str) -> Result<(), String> {
        put_attr_as_type!(self, name, ncid, nc_int64, varid, nc_put_att_longlong)
    }
}

impl PutAttr for u64 {
    fn get_nc_type(&self) -> i32 { nc_uint64 }
    fn put(&self, ncid: i32, varid: i32, name: &str) -> Result<(), String> {
        put_attr_as_type!(self, name, ncid, nc_uint64, varid, nc_put_att_ulonglong)
    }
}

impl PutAttr for f32 {
    fn get_nc_type(&self) -> i32 { nc_float }
    fn put(&self, ncid: i32, varid: i32, name: &str) -> Result<(), String> {
        put_attr_as_type!(self, name, ncid, nc_float, varid, nc_put_att_float)
    }
}

impl PutAttr for f64 {
    fn get_nc_type(&self) -> i32 { nc_double }
    fn put(&self, ncid: i32, varid: i32, name: &str) -> Result<(), String> {
        put_attr_as_type!(self, name, ncid, nc_double, varid, nc_put_att_double)
    }
}

impl PutAttr for String {
    fn get_nc_type(&self) -> i32 { nc_char }
    fn put(&self, ncid: i32, varid: i32, name: &str) -> Result<(), String> {
        let name_c: ffi::CString = ffi::CString::new(name.clone()).unwrap();
        let attr_c: ffi::CString = ffi::CString::new(self.clone()).unwrap();
        let err : i32;
        unsafe {
            let _g = libnetcdf_lock.lock().unwrap();
            err = nc_put_att_text(
                ncid, varid, name_c.as_ptr(), 
                attr_c.to_bytes().len() as u64, attr_c.as_ptr());
        }
        if err != nc_noerr {
            return Err(NC_ERRORS.get(&err).unwrap().clone());
        }
        Ok(())
    }
}

impl Group {
    pub fn add_attribute<T: PutAttr>(&mut self, name: &str, val: T) 
            -> Result<(), String> {
        try!(val.put(self.id, nc_global, name));
        self.attributes.insert(
                name.to_string().clone(),
                Attribute {
                    name: name.to_string().clone(),
                    attrtype: val.get_nc_type(),
                    id: 0, // XXX Should Attribute even keep track of an id?
                    var_id: nc_global,
                    file_id: self.id
                }
            );
        Ok(())
    }

    pub fn add_dimension(&mut self, name: &str, len: u64) 
            -> Result<(), String> {
        let name_c: ffi::CString = ffi::CString::new(name.clone()).unwrap();
        let mut dimid: i32 = 0;
        let err : i32;
        unsafe {
            let _g = libnetcdf_lock.lock().unwrap();
            err = nc_def_dim(self.id, name_c.as_ptr(), len, &mut dimid);
        }
        if err != nc_noerr {
            return Err(NC_ERRORS.get(&err).unwrap().clone());
        }
        self.dimensions.insert(
                name.to_string().clone(),
                Dimension {
                    name: name.to_string().clone(),
                    len: len,
                    id: dimid
                }
            );
        Ok(())
    }

    // TODO this should probably take &Vec<&str> instead of &Vec<String>
    pub fn add_variable<T: PutVar>(
            &mut self, name: &str, dims: &Vec<String>, data: &T) 
                -> Result<(), String>
    {
        let name_c: ffi::CString = ffi::CString::new(name.clone()).unwrap();
        let mut dimids: Vec<i32> = Vec::with_capacity(dims.len());
        let mut var_len : u64 = 1;
        let mut var_dims : Vec<Dimension> = Vec::with_capacity(dims.len());
        let nctype = data.get_nc_type();
        for dim_name in dims {
            if !self.dimensions.contains_key(dim_name) {
                return Err("Invalid dimension name".to_string());
            }
            var_dims.push(self.dimensions.get(dim_name).unwrap().clone());
        }
        for dim in &var_dims {
            dimids.push(dim.id);
            var_len *= dim.len;
        }
        if data.len() != (var_len as usize) {
            return Err("Vec length must match product of all dims".to_string());
        }
        let mut varid: i32 = 0;
        let err : i32;
        unsafe {
            let _g = libnetcdf_lock.lock().unwrap();
            err = nc_def_var(self.id, name_c.as_ptr(), nctype,
                                dims.len() as i32, dimids.as_ptr(), &mut varid);
        }
        if err != nc_noerr {
            return Err(NC_ERRORS.get(&err).unwrap().clone());
        }
        try!(data.put(self.id, varid));
        self.variables.insert(
            name.to_string().clone(),
            Variable {
                name: name.to_string().clone(),
                attributes: HashMap::new(),
                dimensions: var_dims,
                vartype: nctype,
                id: varid,
                len: var_len,
                file_id: self.id
            }
        );
        Ok(())
    }
}

fn init_sub_groups(grp_id: i32, sub_groups: &mut HashMap<String, Group>,
                   parent_dims: &HashMap<String, Dimension>) {
    let mut ngrps = 0i32;
    // Max number of groups in a file is only limited by i32 max (32767)...
    // allocating a vec this size is inefficient but there's no obvious way
    // to query the number of groups beforehand!
    // http://www.unidata.ucar.edu/software/netcdf/docs/group__groups.html#details
    let mut grpids : Vec<i32> = Vec::with_capacity(nc_max_int as usize);
    unsafe {
        let _g = libnetcdf_lock.lock().unwrap();

        // number of groups and grp id's
        let err = nc_inq_grps(grp_id, &mut ngrps, grpids.as_mut_ptr());
        assert_eq!(err, nc_noerr);
        grpids.set_len(ngrps as usize);
    }
    for i_grp in 0..ngrps {
        let mut namelen = 0u64;
        let c_str: &ffi::CStr;
        unsafe {
            let _g = libnetcdf_lock.lock().unwrap();
            // name length
            let err = nc_inq_grpname_len(grpids[i_grp as usize], &mut namelen);
            assert_eq!(err, nc_noerr);
            // name
            let mut buf_vec = vec![0i8; (namelen+1) as usize];
            let buf_ptr : *mut i8 = buf_vec.as_mut_ptr();
            let err = nc_inq_grpname(grpids[i_grp as usize], buf_ptr);
            assert_eq!(err, nc_noerr);
            c_str = ffi::CStr::from_ptr(buf_ptr);
        }
        let str_buf: String = string_from_c_str(c_str);

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

    init_attributes(&mut grp.attributes, grp.id, nc_global, -1);
    
    init_variables(&mut grp.variables, grp.id, &grp.dimensions);

    init_sub_groups(grp.id, &mut grp.sub_groups, &grp.dimensions);
}
