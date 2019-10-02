use super::error;
use super::group::Group;
use super::LOCK;
use netcdf_sys::*;
use super::HashMap;
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

impl std::ops::Deref for File {
    type Target = Group;
    fn deref(&self) -> &Self::Target {
        &self.root
    }
}

impl std::ops::DerefMut for File {
    fn deref_mut(&mut self) -> &mut Self::Target {
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
            variables: Default::default(),
            attributes: Default::default(),
            parent_dimensions: Default::default(),
            dimensions: Default::default(),
            groups: Default::default(),
        };
        Ok(File {
            ncid,
            name: data_path.file_name().unwrap().to_string_lossy().to_string(),
            root,
        })
    }
}

#[cfg(feature = "memory")]
/// The memory mapped file is kept in this structure to keep the
/// lifetime of the buffer longer than the file.
///
/// Access the [`File`] through the traits `Deref` or `DerefMut`
pub struct MemFile<'a> {
    file: File,
    _buffer: std::marker::PhantomData<&'a [u8]>,
}

#[cfg(feature = "memory")]
impl<'a> std::ops::Deref for MemFile<'a> {
    type Target = File;
    fn deref(&self) -> &Self::Target {
        &self.file
    }
}
#[cfg(feature = "memory")]
impl<'a> std::ops::DerefMut for MemFile<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.file
    }
}

#[cfg(feature = "memory")]
impl<'a> MemFile<'a> {
    pub fn new(name: Option<&str>, mem: &'a [u8]) -> error::Result<Self> {
        let cstr = std::ffi::CString::new(name.unwrap_or("/")).unwrap();
        let mut ncid = 0;
        let err;
        unsafe {
            let _l = LOCK.lock().unwrap();
            err = nc_open_mem(
                cstr.as_ptr(),
                NC_NOWRITE,
                mem.len(),
                mem.as_ptr() as *const u8 as *mut _,
                &mut ncid,
            );
        }
        if err != NC_NOERR {
            return Err(err.into());
        }

        let root = parse_file(ncid)?;

        Ok(Self {
            file: File {
                name: name.unwrap_or("").to_string(),
                ncid,
                root,
            },
            _buffer: std::marker::PhantomData,
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

fn get_group_dimensions(
    ncid: nc_type,
    unlimited_dims: &[nc_type],
) -> error::Result<HashMap<String, Dimension>> {
    let mut ndims: nc_type = 0;
    let err;
    unsafe {
        err = nc_inq_dimids(ncid, &mut ndims, std::ptr::null_mut(), 0);
    }
    if err != NC_NOERR {
        return Err(err.into());
    }
    if ndims == 0 {
        return Ok(HashMap::new());
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

        let len = if unlimited_dims.contains(&dimid) {
            None
        } else {
            Some(unsafe { core::num::NonZeroUsize::new_unchecked(len) })
        };
        dimensions.insert(
            name.clone(),
            Dimension {
                ncid,
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
    if natts == 0 {
        return Ok(HashMap::new());
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
        };
        attributes.insert(name, a);
    }

    Ok(attributes)
}

fn get_dimensions_of_var(
    ncid: nc_type,
    varid: nc_type,
    unlimited_dims: &[nc_type],
) -> error::Result<Vec<Dimension>> {
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
    if ndims == 0 {
        return Ok(Vec::new());
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

        let unlimited = unlimited_dims.contains(&dimid);
        let len = if unlimited {
            None
        } else {
            Some(unsafe { core::num::NonZeroUsize::new_unchecked(dimlen) })
        };
        let d = Dimension {
            ncid,
            name,
            len,
            id: dimid,
        };
        dimensions.push(d);
    }

    Ok(dimensions)
}

use super::Variable;
fn get_variables(
    ncid: nc_type,
    unlimited_dims: &[nc_type],
) -> error::Result<HashMap<String, Variable>> {
    let err;
    let mut nvars = 0;
    unsafe {
        err = nc_inq_varids(ncid, &mut nvars, std::ptr::null_mut());
    }
    if err != NC_NOERR {
        return Err(err.into());
    }
    if nvars == 0 {
        return Ok(HashMap::new());
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
        let dimensions = get_dimensions_of_var(ncid, varid, unlimited_dims)?;

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

fn get_groups(
    ncid: nc_type,
    parent_dim: &[HashMap<String, Dimension>],
) -> error::Result<HashMap<String, Group>> {
    let err;
    let mut ngroups = 0;

    unsafe {
        err = nc_inq_grps(ncid, &mut ngroups, std::ptr::null_mut());
    }
    if err != NC_NOERR {
        return Err(err.into());
    }
    if ngroups == 0 {
        return Ok(HashMap::new());
    }
    let err;
    let mut grpids = vec![0; ngroups as usize];
    unsafe {
        err = nc_inq_grps(ncid, std::ptr::null_mut(), grpids.as_mut_ptr());
    }
    if err != NC_NOERR {
        return Err(err.into());
    }
    let mut groups = HashMap::with_capacity(ngroups as usize);
    for grpid in grpids.into_iter() {
        let unlim_dims = get_unlimited_dimensions(grpid)?;
        let dimensions = get_group_dimensions(grpid, &unlim_dims)?;
        let variables = get_variables(grpid, &unlim_dims)?;
        let mut parent_dimensions = parent_dim.to_vec();
        parent_dimensions.push(dimensions.clone());
        let subgroups = get_groups(grpid, &parent_dimensions)?;
        let attributes = get_attributes(grpid, NC_GLOBAL)?;

        let mut cname = [0; NC_MAX_NAME as usize + 1];
        let err;
        unsafe {
            err = nc_inq_grpname(grpid, cname.as_mut_ptr());
        }
        if err != NC_NOERR {
            return Err(err.into());
        }
        let name = unsafe { std::ffi::CStr::from_ptr(cname.as_ptr()) }
            .to_string_lossy()
            .to_string();

        let g = Group {
            name: name.clone(),
            ncid,
            grpid: Some(grpid),
            attributes,
            parent_dimensions: parent_dim.to_vec(),
            dimensions,
            variables,
            groups: subgroups,
        };
        groups.insert(name, g);
    }

    Ok(groups)
}

fn get_unlimited_dimensions(ncid: nc_type) -> error::Result<Vec<nc_type>> {
    let err;
    let mut nunlim = 0;
    unsafe {
        err = nc_inq_unlimdims(ncid, &mut nunlim, std::ptr::null_mut());
    }
    if err != NC_NOERR {
        return Err(err.into());
    }
    let mut uldim = vec![0; nunlim as usize];
    let err;
    unsafe {
        err = nc_inq_unlimdims(ncid, std::ptr::null_mut(), uldim.as_mut_ptr());
    }
    if err != NC_NOERR {
        return Err(err.into());
    }
    Ok(uldim)
}

fn parse_file(ncid: nc_type) -> error::Result<Group> {
    let _l = LOCK.lock().unwrap();

    let unlimited_dimensions = get_unlimited_dimensions(ncid)?;
    let dimensions = get_group_dimensions(ncid, &unlimited_dimensions)?;

    let attributes = get_attributes(ncid, NC_GLOBAL)?;

    let variables = get_variables(ncid, &unlimited_dimensions)?;

    // Empty parent group
    let groups = get_groups(ncid, &[])?;

    Ok(Group {
        ncid,
        grpid: None,
        name: "root".into(),
        parent_dimensions: Default::default(),
        dimensions,
        attributes,
        variables,
        groups,
    })
}
