use std::ffi;
use std::path;
use std::collections::HashMap;
use netcdf_sys::*;
use group::{init_group, Group};
use NC_ERRORS;

pub struct File {
    pub id: i32,
    pub name: String,
    pub root: Group,
}

pub fn open(file: &str) -> Result<File, String> {
    let data_path = path::Path::new(file);
    let f = ffi::CString::new(data_path.to_str().unwrap()).unwrap();
    let mut ncid : i32 = -999999i32;
    let err : i32;
    unsafe {
        let _g = libnetcdf_lock.lock().unwrap();
        err = nc_open(f.as_ptr(), nc_nowrite, &mut ncid);
    }
    if err != nc_noerr {
        return Err(NC_ERRORS.get(&err).unwrap().clone());
    }
    let mut root = Group {
            name: "root".to_string(),
            id: ncid,
            variables: HashMap::new(),
            attributes: HashMap::new(),
            dimensions: HashMap::new(),
            sub_groups: HashMap::new(),
        };
    init_group(&mut root);
    Ok(File {
        id: ncid, 
        name: file.to_string(),
        root: root,
    })
}

pub fn create(file: &str) -> Result<File, String> {
    let data_path = path::Path::new(file);
    let f = ffi::CString::new(data_path.to_str().unwrap()).unwrap();
    let mut ncid : i32 = -999999i32;
    let err : i32;
    unsafe {
        let _g = libnetcdf_lock.lock().unwrap();
        err = nc_create(f.as_ptr(), nc_netcdf4, &mut ncid);
    }
    if err != nc_noerr {
        return Err(NC_ERRORS.get(&err).unwrap().clone());
    }
    let root = Group {
            name: "root".to_string(),
            id: ncid,
            variables: HashMap::new(),
            attributes: HashMap::new(),
            dimensions: HashMap::new(),
            sub_groups: HashMap::new(),
        };
    Ok(File {
        id: ncid, 
        name: file.to_string(),
        root: root,
    })
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            let _g = libnetcdf_lock.lock().unwrap();
            let err = nc_close(self.id);
            assert_eq!(err, nc_noerr);
        }
    }
}

