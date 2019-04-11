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

/// Open a netCDF file in read only mode.
pub fn open(file: &str) -> Result<File, String> {
    let data_path = path::Path::new(file);
    let f = ffi::CString::new(data_path.to_str().unwrap()).unwrap();
    let mut ncid : i32 = -999999i32;
    let err : i32;
    unsafe {
        let _g = libnetcdf_lock.lock().unwrap();
        err = nc_open(f.as_ptr(), NC_NOWRITE, &mut ncid);
    }
    if err != NC_NOERR {
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


/// Open a netCDF file held in memory as `&[u8]`.
pub fn open_from_bytes<S: Into<String>>(mut file: Vec<u8>, name: S) -> Result<File, String> {
    let mut ncid : i32 = -999999i32;
    let err : i32;
    let string_name: String = name.into();
    let f = ffi::CString::new(string_name.as_str()).unwrap();

    use std::mem;
    use std::os::raw::c_void;

    file.shrink_to_fit();
    let size = file.len() as u64;
    let file = Box::new(file);

    unsafe {
        let _g = libnetcdf_lock.lock().unwrap();
        err = nc_open_mem(f.as_ptr(), NC_NOWRITE, size, Box::into_raw(file) as *mut c_void, &mut ncid);
    }
    if err != NC_NOERR {
        eprintln!("Returing error!: {:?}", &err);
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
        name: string_name,
        root: root,
    })
}


/// Open a netCDF file in append mode (read/write).
/// The file must already exist.
pub fn append(file: &str) -> Result<File, String> {
    let data_path = path::Path::new(file);
    let f = ffi::CString::new(data_path.to_str().unwrap()).unwrap();
    let mut ncid : i32 = -999999i32;
    let err : i32;
    unsafe {
        let _g = libnetcdf_lock.lock().unwrap();
        err = nc_open(f.as_ptr(), NC_WRITE, &mut ncid);
    }
    if err != NC_NOERR {
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

/// Open a netCDF file in creation mode (write only).
pub fn create(file: &str) -> Result<File, String> {
    let data_path = path::Path::new(file);
    let f = ffi::CString::new(data_path.to_str().unwrap()).unwrap();
    let mut ncid : i32 = -999999i32;
    let err : i32;
    unsafe {
        let _g = libnetcdf_lock.lock().unwrap();
        err = nc_create(f.as_ptr(), NC_NETCDF4, &mut ncid);
    }
    if err != NC_NOERR {
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

impl File{
    fn close(&mut self) {
        unsafe {
            let _g = libnetcdf_lock.lock().unwrap();
            let err = nc_close(self.id);
            assert_eq!(err, NC_NOERR);
        }
    }
}

impl Drop for File {
    fn drop(&mut self) {
        // Automatically close file when it goes out of scope
        self.close();
    }
}

