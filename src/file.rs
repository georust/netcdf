use super::error;
use super::group::Group;
use super::LOCK;
use netcdf_sys::*;
use std::collections::HashMap;
use std::ffi::CString;
use std::path;

#[derive(Debug)]
pub struct File {
    pub(crate) ncid: nc_type,
    pub(crate) name: String,
    pub(crate) root: Group,
}

impl File {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn root(&self) -> &Group {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut Group {
        &mut self.root
    }
}

impl File {
    /// Open a netCDF file in read only mode.
    pub fn open<P>(file: P) -> error::Result<File>
    where
        P: AsRef<path::Path>,
    {
        let data_path = file.as_ref();
        let f = CString::new(data_path.to_str().unwrap()).unwrap();
        let mut ncid: nc_type = -1;
        let err: nc_type;
        unsafe {
            let _g = LOCK.lock().unwrap();
            err = nc_open(f.as_ptr(), NC_NOWRITE, &mut ncid);
        }
        if err != NC_NOERR {
            return Err(err.into());
        }

        let root = Group {
            name: "".into(),
            ncid,
            grpid: None,
            variables: HashMap::new(),
            attributes: HashMap::new(),
            dimensions: HashMap::new(),
            sub_groups: HashMap::new(),
        };

        Ok(File {
            ncid,
            name: data_path.to_string_lossy().into_owned(),
            root,
        })
    }
    /// Open a netCDF file in append mode (read/write).
    /// The file must already exist.
    pub fn append<P>(file: P) -> error::Result<File>
    where
        P: AsRef<path::Path>,
    {
        let data_path = file.as_ref();
        let f = CString::new(data_path.to_str().unwrap()).unwrap();
        let mut ncid: nc_type = -1;
        let err: nc_type;
        unsafe {
            let _g = LOCK.lock().unwrap();
            err = nc_open(f.as_ptr(), NC_WRITE, &mut ncid);
        }
        if err != NC_NOERR {
            return Err(err.into());
        }
        let root = Group {
            name: "root".to_string(),
            ncid,
            grpid: None,
            variables: HashMap::new(),
            attributes: HashMap::new(),
            dimensions: HashMap::new(),
            sub_groups: HashMap::new(),
        };
        Ok(File {
            ncid,
            name: data_path.to_string_lossy().into_owned(),
            root,
        })
    }
    /// Open a netCDF file in creation mode (write only).
    pub fn create<P>(file: P) -> error::Result<File>
    where
        P: AsRef<path::Path>,
    {
        let data_path = file.as_ref();
        let f = CString::new(data_path.to_str().unwrap()).unwrap();
        let mut ncid: nc_type = -1;
        let err: nc_type;
        unsafe {
            let _g = LOCK.lock().unwrap();
            err = nc_create(f.as_ptr(), NC_NETCDF4, &mut ncid);
        }
        if err != NC_NOERR {
            return Err(err.into());
        }
        let root = Group {
            name: "root".to_string(),
            ncid,
            grpid: None,
            variables: HashMap::new(),
            attributes: HashMap::new(),
            dimensions: HashMap::new(),
            sub_groups: HashMap::new(),
        };
        Ok(File {
            ncid,
            name: data_path.to_string_lossy().into_owned(),
            root,
        })
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            let _g = LOCK.lock().unwrap();
            let err = nc_close(self.ncid);
            assert_eq!(err, NC_NOERR);
        }
    }
}
