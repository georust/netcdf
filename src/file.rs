use group::{init_group, Group};
use netcdf_sys::*;
use std::collections::HashMap;
use std::ffi;
use std::path;
use NC_ERRORS;

#[derive(Debug)]
pub struct File {
    pub id: nc_type,
    pub name: String,
    pub root: Group,
}

/// Open a netCDF file in read only mode.
pub fn open<P>(file: P) -> Result<File, String>
where
    P: AsRef<path::Path>,
{
    let data_path = file.as_ref();
    let f = ffi::CString::new(data_path.to_str().unwrap()).unwrap();
    let mut ncid: nc_type = -999999;
    let err: nc_type;
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
        name: data_path.to_string_lossy().into_owned(),
        root: root,
    })
}

/// Open a netCDF file in append mode (read/write).
/// The file must already exist.
pub fn append<P>(file: P) -> Result<File, String>
where
    P: AsRef<path::Path>,
{
    let data_path = file.as_ref();
    let f = ffi::CString::new(data_path.to_str().unwrap()).unwrap();
    let mut ncid: nc_type = -999999;
    let err: nc_type;
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
        name: data_path.to_string_lossy().into_owned(),
        root: root,
    })
}

/// Open a netCDF file in creation mode (write only).
pub fn create<P>(file: P) -> Result<File, String>
where
    P: AsRef<path::Path>,
{
    let data_path = file.as_ref();
    let f = ffi::CString::new(data_path.to_str().unwrap()).unwrap();
    let mut ncid: nc_type = -999999;
    let err: nc_type;
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
        name: data_path.to_string_lossy().into_owned(),
        root: root,
    })
}

impl File {
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
