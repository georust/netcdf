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

        let root = parse_file(ncid)?;

        Ok(File {
            ncid,
            name: data_path.file_name().unwrap().to_string_lossy().to_string(),
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

        let root = parse_file(ncid)?;

        Ok(File {
            ncid,
            name: data_path.file_name().unwrap().to_string_lossy().to_string(),
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
            err = nc_create(f.as_ptr(), NC_NETCDF4 | NC_NOCLOBBER, &mut ncid);
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
            name: data_path.file_name().unwrap().to_string_lossy().to_string(),
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

use super::dimension::Dimension;

fn get_group_dimensions(ncid: nc_type) -> error::Result<HashMap<String, Dimension>> {
    let mut ndims: nc_type = 0;
    let err;
    unsafe {
        err = nc_inq_dimids(ncid, &mut ndims, std::ptr::null_mut(), 0);
    }
    if err != NC_NOERR {
        return Err(err.into());
    }
    let mut dimids = vec![0 as nc_type; ndims as usize];
    let err;
    unsafe {
        err = nc_inq_dimids(ncid, std::ptr::null_mut(), dimids.as_mut_ptr(), 0);
    }
    if err != NC_NOERR {
        return Err(err.into());
    }

    let mut dimensions = HashMap::with_capacity(ndims as _);
    for dimid in dimids.into_iter() {
        let mut buf = [0u8; NC_MAX_NAME as usize + 1];
        let mut len = 0;
        let err;
        unsafe {
            err = nc_inq_dim(ncid, dimid as _, buf.as_mut_ptr() as *mut _, &mut len);
        }
        if err != NC_NOERR {
            return Err(err.into());
        }
        let name: Vec<_> = buf
            .iter()
            .take_while(|x| **x != 0)
            .map(|x| *x as u8)
            .collect();
        let name = String::from_utf8(name).unwrap();
        dimensions.insert(
            name.clone(),
            Dimension {
                name,
                len,
                id: dimid,
            },
        );
    }

    Ok(dimensions)
}

use super::attribute::Attribute;
fn get_attributes(ncid: nc_type, varid: nc_type) -> error::Result<HashMap<String, Attribute>> {
    let err;
    let mut natts = 0;
    unsafe {
        err = nc_inq_varnatts(ncid, varid, &mut natts);
    }
    if err != NC_NOERR {
        return Err(err.into());
    }
    let mut attributes = HashMap::with_capacity(natts as _);
    for i in 0..natts {
        let mut buf = [0u8; NC_MAX_NAME as usize + 1];
        let err = unsafe { nc_inq_attname(ncid, varid, i, buf.as_mut_ptr() as *mut _) };

        if err != NC_NOERR {
            return Err(err.into());
        }
        let name: Vec<_> = buf
            .iter()
            .take_while(|x| **x != 0)
            .map(|x| *x as u8)
            .collect();
        let name = String::from_utf8(name).unwrap();
        let a = Attribute {
            name: name.clone(),
            ncid,
            varid,
            value: None,
        };
        attributes.insert(name, a);
    }

    Ok(attributes)
}

fn get_dimensions_of_var(ncid: nc_type, varid: nc_type) -> error::Result<Vec<Dimension>> {
    let mut ndims = 0;
    let err;
    unsafe {
        err = nc_inq_var(
            ncid,
            varid,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut ndims,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
    }
    if err != NC_NOERR {
        return Err(err.into());
    }
    let mut dimids = vec![0; ndims as usize];
    let err;
    unsafe {
        err = nc_inq_var(
            ncid,
            varid,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            dimids.as_mut_ptr(),
            std::ptr::null_mut(),
        );
    }
    if err != NC_NOERR {
        return Err(err.into());
    }

    let mut dimensions = Vec::with_capacity(ndims as usize);
    for dimid in dimids.into_iter() {
        let mut name = vec![0u8; NC_MAX_NAME as usize + 1];
        let mut dimlen = 0;
        let err;
        unsafe {
            err = nc_inq_dim(ncid, dimid, name.as_mut_ptr() as *mut _, &mut dimlen);
        }

        if err != NC_NOERR {
            return Err(err.into());
        }

        let cstr = std::ffi::CString::new(
            name.into_iter()
                .take_while(|x| *x != 0)
                .collect::<Vec<u8>>(),
        )
        .unwrap();
        let name = cstr.to_string_lossy().into_owned();

        let d = Dimension {
            name,
            len: dimlen,
            id: dimid,
        };
        dimensions.push(d);
    }

    Ok(dimensions)
}

use super::Variable;
fn get_variables(ncid: nc_type) -> error::Result<HashMap<String, Variable>> {
    let err;
    let mut nvars = 0;
    unsafe {
        err = nc_inq_varids(ncid, &mut nvars, std::ptr::null_mut());
    }
    if err != NC_NOERR {
        return Err(err.into());
    }
    let mut varids = vec![0; nvars as usize];
    let err;
    unsafe {
        err = nc_inq_varids(ncid, std::ptr::null_mut(), varids.as_mut_ptr());
    }
    if err != NC_NOERR {
        return Err(err.into());
    }
    let mut variables = HashMap::with_capacity(nvars as usize);
    for varid in varids.into_iter() {
        let mut name = vec![0u8; NC_MAX_NAME as usize + 1];
        let mut vartype = 0;
        let err;
        unsafe {
            err = nc_inq_var(
                ncid,
                varid,
                name.as_mut_ptr() as *mut _,
                &mut vartype,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
        }
        if err != NC_NOERR {
            return Err(err.into());
        }
        let attributes = get_attributes(ncid, varid)?;
        let dimensions = get_dimensions_of_var(ncid, varid)?;

        let cstr = std::ffi::CString::new(
            name.into_iter()
                .take_while(|x| *x != 0)
                .collect::<Vec<u8>>(),
        )
        .unwrap();
        let name = cstr.to_string_lossy().into_owned();
        let v = Variable {
            ncid,
            varid,
            dimensions,
            name: name.clone(),
            attributes,
            vartype,
        };

        variables.insert(name, v);
    }

    Ok(variables)
}

fn parse_file(ncid: nc_type) -> error::Result<Group> {
    let _l = LOCK.lock().unwrap();

    let dimensions = get_group_dimensions(ncid)?;

    let attributes = get_attributes(ncid, NC_GLOBAL)?;

    let variables = get_variables(ncid)?;

    let sub_groups = HashMap::new();

    Ok(Group {
        ncid,
        grpid: None,
        name: "root".into(),
        dimensions,
        attributes,
        variables,
        sub_groups,
    })
}
