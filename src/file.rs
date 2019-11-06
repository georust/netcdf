//! Open, create, and append netcdf files

#![allow(clippy::similar_names)]
use super::error;
use super::group::Group;
use super::HashMap;
use super::LOCK;
use netcdf_sys::*;
use std::cell::UnsafeCell;
use std::convert::TryInto;
use std::ffi::CString;
use std::path;
use std::rc::Rc;

/// Container for netcdf type
#[derive(Debug)]
pub struct File {
    pub(crate) ncid: nc_type,
    pub(crate) name: String,
    pub(crate) root: Rc<UnsafeCell<Group>>,
}

impl File {
    /// Current name of the file. This name sometimes gives the name
    /// of the file used to open it, or some arbitrary name when
    /// opened through a memory buffer
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Main entrypoint for interacting with the netcdf file. Also accessible
    /// through the `Deref` trait on `File`
    pub fn root(&self) -> &Group {
        unsafe { &*self.root.get() }
    }

    /// Mutable access to the root group
    pub fn root_mut(&mut self) -> &mut Group {
        unsafe { &mut *self.root.get() }
    }
}

impl std::ops::Deref for File {
    type Target = Group;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.root.get() }
    }
}

impl std::ops::DerefMut for File {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.root.get() }
    }
}

impl File {
    #[allow(clippy::doc_markdown)]
    /// Open a netCDF file in read only mode.
    pub fn open(path: &path::Path) -> error::Result<Self> {
        let f = CString::new(path.to_str().unwrap()).unwrap();
        let mut ncid: nc_type = -1;
        unsafe {
            let _g = LOCK.lock().unwrap();
            error::checked(nc_open(f.as_ptr(), NC_NOWRITE, &mut ncid))?;
        }

        let root = parse_file(ncid)?;

        Ok(Self {
            ncid,
            name: path.file_name().unwrap().to_string_lossy().to_string(),
            root,
        })
    }
    #[allow(clippy::doc_markdown)]
    /// Open a netCDF file in append mode (read/write).
    /// The file must already exist.
    pub fn append(path: &path::Path) -> error::Result<Self> {
        let f = CString::new(path.to_str().unwrap()).unwrap();
        let mut ncid: nc_type = -1;
        unsafe {
            let _g = LOCK.lock().unwrap();
            error::checked(nc_open(f.as_ptr(), NC_WRITE, &mut ncid))?;
        }

        let root = parse_file(ncid)?;

        Ok(Self {
            ncid,
            name: path.file_name().unwrap().to_string_lossy().to_string(),
            root,
        })
    }
    #[allow(clippy::doc_markdown)]
    /// Open a netCDF file in creation mode.
    ///
    /// Will overwrite existing file if any
    pub fn create(path: &path::Path) -> error::Result<Self> {
        let f = CString::new(path.to_str().unwrap()).unwrap();
        let mut ncid: nc_type = -1;
        unsafe {
            let _g = LOCK.lock().unwrap();
            error::checked(nc_create(f.as_ptr(), NC_NETCDF4 | NC_CLOBBER, &mut ncid))?;
        }

        let root = Rc::new(UnsafeCell::new(Group {
            name: "root".to_string(),
            ncid,
            grpid: None,
            variables: HashMap::default(),
            attributes: Vec::default(),
            dimensions: HashMap::default(),
            groups: HashMap::default(),
            parent: None,
            this: None,
        }));
        {
            let rootref = Some(Rc::downgrade(&root));
            let root = unsafe { &mut *root.get() };
            root.this = rootref;
        }
        Ok(Self {
            ncid,
            name: path.file_name().unwrap().to_string_lossy().to_string(),
            root,
        })
    }
}

#[cfg(feature = "memory")]
/// The memory mapped file is kept in this structure to keep the
/// lifetime of the buffer longer than the file.
///
/// Access the [`File`] through the `Deref` trait,
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let buffer = &[0, 42, 1, 2];
/// let file = &netcdf::MemFile::new(None, buffer)?;
///
/// let variables = file.variables();
/// # Ok(()) }
/// ```
#[allow(clippy::module_name_repetitions)]
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
impl<'a> MemFile<'a> {
    /// Open a file from the given buffer
    pub fn new(name: Option<&str>, mem: &'a [u8]) -> error::Result<Self> {
        let cstr = std::ffi::CString::new(name.unwrap_or("/")).unwrap();
        let mut ncid = 0;
        unsafe {
            let _l = LOCK.lock().unwrap();
            error::checked(nc_open_mem(
                cstr.as_ptr(),
                NC_NOWRITE,
                mem.len(),
                mem.as_ptr() as *const u8 as *mut _,
                &mut ncid,
            ))?;
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
            // Can't really do much with an error here
            let _err = error::checked(nc_close(self.ncid));
        }
    }
}

use super::dimension::Dimension;

fn get_group_dimensions(
    ncid: nc_type,
    unlimited_dims: &[nc_type],
) -> error::Result<HashMap<String, Dimension>> {
    let mut ndims: nc_type = 0;
    unsafe {
        error::checked(nc_inq_dimids(ncid, &mut ndims, std::ptr::null_mut(), 0))?;
    }

    if ndims == 0 {
        return Ok(HashMap::new());
    }
    let mut dimids = vec![0 as nc_type; ndims.try_into()?];
    unsafe {
        error::checked(nc_inq_dimids(
            ncid,
            std::ptr::null_mut(),
            dimids.as_mut_ptr(),
            0,
        ))?;
    }

    let mut dimensions = HashMap::with_capacity(ndims.try_into()?);
    let mut buf = [0_u8; NC_MAX_NAME as usize + 1];
    for dimid in dimids {
        for i in buf.iter_mut() {
            *i = 0
        }
        let mut len = 0;
        unsafe {
            error::checked(nc_inq_dim(
                ncid,
                dimid as _,
                buf.as_mut_ptr() as *mut _,
                &mut len,
            ))?;
        }

        let zero_pos = buf
            .iter()
            .position(|&x| x == 0)
            .unwrap_or_else(|| buf.len());
        let name = String::from(String::from_utf8_lossy(&buf[..zero_pos]));

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
fn get_attributes(ncid: nc_type, varid: nc_type) -> error::Result<Vec<Attribute>> {
    let mut natts = 0;
    unsafe {
        error::checked(nc_inq_varnatts(ncid, varid, &mut natts))?;
    }
    if natts == 0 {
        return Ok(Vec::new());
    }
    let mut attributes = Vec::with_capacity(natts.try_into()?);
    let mut buf = [0_u8; NC_MAX_NAME as usize + 1];
    for i in 0..natts {
        for i in buf.iter_mut() {
            *i = 0;
        }
        unsafe { error::checked(nc_inq_attname(ncid, varid, i, buf.as_mut_ptr() as *mut _))? };

        let zero_pos = buf
            .iter()
            .position(|&x| x == 0)
            .unwrap_or_else(|| buf.len());
        let name = String::from(String::from_utf8_lossy(&buf[..zero_pos]));
        let a = Attribute {
            name: name.clone(),
            ncid,
            varid,
        };
        attributes.push(a);
    }

    Ok(attributes)
}

fn get_dimensions_of_var(
    ncid: nc_type,
    varid: nc_type,
    unlimited_dims: &[nc_type],
) -> error::Result<Vec<Dimension>> {
    let mut ndims = 0;
    unsafe {
        error::checked(nc_inq_var(
            ncid,
            varid,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut ndims,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ))?;
    }
    if ndims == 0 {
        return Ok(Vec::new());
    }
    let mut dimids = vec![0; ndims.try_into()?];
    unsafe {
        error::checked(nc_inq_var(
            ncid,
            varid,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            dimids.as_mut_ptr(),
            std::ptr::null_mut(),
        ))?;
    }

    let mut dimensions = Vec::with_capacity(ndims.try_into()?);
    let mut name = [0_u8; NC_MAX_NAME as usize + 1];
    for dimid in dimids {
        for i in name.iter_mut() {
            *i = 0;
        }
        let mut dimlen = 0;
        unsafe {
            error::checked(nc_inq_dim(
                ncid,
                dimid,
                name.as_mut_ptr() as *mut _,
                &mut dimlen,
            ))?;
        }

        let zero_pos = name
            .iter()
            .position(|&x| x == 0)
            .unwrap_or_else(|| name.len());
        let name = String::from(String::from_utf8_lossy(&name[..zero_pos]));

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
    let mut nvars = 0;
    unsafe {
        error::checked(nc_inq_varids(ncid, &mut nvars, std::ptr::null_mut()))?;
    }
    if nvars == 0 {
        return Ok(HashMap::new());
    }
    let mut varids = vec![0; nvars.try_into()?];
    unsafe {
        error::checked(nc_inq_varids(
            ncid,
            std::ptr::null_mut(),
            varids.as_mut_ptr(),
        ))?;
    }

    let mut variables = HashMap::with_capacity(nvars.try_into()?);
    let mut name = [0_u8; NC_MAX_NAME as usize + 1];
    for varid in varids {
        for i in name.iter_mut() {
            *i = 0;
        }
        let mut vartype = 0;
        unsafe {
            error::checked(nc_inq_var(
                ncid,
                varid,
                name.as_mut_ptr() as *mut _,
                &mut vartype,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ))?;
        }
        let attributes = get_attributes(ncid, varid)?;
        let dimensions = get_dimensions_of_var(ncid, varid, unlimited_dims)?;

        let zero_pos = name
            .iter()
            .position(|&x| x == 0)
            .unwrap_or_else(|| name.len());
        let name = String::from(String::from_utf8_lossy(&name[..zero_pos]));

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
    parent_dim: &[&HashMap<String, Dimension>],
    parent: &Rc<UnsafeCell<Group>>,
) -> error::Result<HashMap<String, Rc<UnsafeCell<Group>>>> {
    let mut ngroups = 0;

    unsafe {
        error::checked(nc_inq_grps(ncid, &mut ngroups, std::ptr::null_mut()))?;
    }
    if ngroups == 0 {
        return Ok(HashMap::new());
    }
    let mut grpids = vec![0; ngroups.try_into()?];
    unsafe {
        error::checked(nc_inq_grps(ncid, std::ptr::null_mut(), grpids.as_mut_ptr()))?;
    }

    let mut groups = HashMap::with_capacity(ngroups.try_into()?);
    let mut cname = [0; NC_MAX_NAME as usize + 1];
    for grpid in grpids {
        let unlim_dims = get_unlimited_dimensions(grpid)?;
        let dimensions = get_group_dimensions(grpid, &unlim_dims)?;
        let variables = get_variables(grpid, &unlim_dims)?;
        let mut parent_dimensions = parent_dim.to_vec();
        parent_dimensions.push(&dimensions);
        let attributes = get_attributes(grpid, NC_GLOBAL)?;

        for i in cname.iter_mut() {
            *i = 0;
        }
        unsafe {
            error::checked(nc_inq_grpname(grpid, cname.as_mut_ptr()))?;
        }

        let name = unsafe { std::ffi::CStr::from_ptr(cname.as_ptr()) }
            .to_string_lossy()
            .to_string();

        let g = Rc::new(UnsafeCell::new(Group {
            name: name.clone(),
            ncid,
            grpid: Some(grpid),
            attributes,
            dimensions: dimensions.clone(),
            variables,
            groups: HashMap::default(),
            parent: Some(Rc::downgrade(parent)),
            this: None,
        }));

        let subgroups = get_groups(grpid, &parent_dimensions, &g)?;
        let refcell = Rc::downgrade(&g);
        {
            let g = unsafe { &mut *g.get() };
            g.this = Some(refcell);
            g.groups = subgroups;
        }

        groups.insert(name, g);
    }

    Ok(groups)
}

fn get_unlimited_dimensions(ncid: nc_type) -> error::Result<Vec<nc_type>> {
    let mut nunlim = 0;
    unsafe {
        error::checked(nc_inq_unlimdims(ncid, &mut nunlim, std::ptr::null_mut()))?;
    }

    let mut uldim = vec![0; nunlim.try_into()?];
    unsafe {
        error::checked(nc_inq_unlimdims(
            ncid,
            std::ptr::null_mut(),
            uldim.as_mut_ptr(),
        ))?;
    }
    Ok(uldim)
}

fn parse_file(ncid: nc_type) -> error::Result<Rc<UnsafeCell<Group>>> {
    let _l = LOCK.lock().unwrap();

    let unlimited_dimensions = get_unlimited_dimensions(ncid)?;
    let dimensions = get_group_dimensions(ncid, &unlimited_dimensions)?;

    let attributes = get_attributes(ncid, NC_GLOBAL)?;

    let variables = get_variables(ncid, &unlimited_dimensions)?;

    let g = Rc::new(UnsafeCell::new(Group {
        ncid,
        grpid: None,
        name: "root".into(),
        dimensions: dimensions.clone(),
        attributes,
        variables,
        groups: HashMap::default(),
        parent: None,
        this: None,
    }));
    let thisref = Some(Rc::downgrade(&g));
    let groups = get_groups(ncid, &[&dimensions], &g)?;
    {
        let g = unsafe { &mut *g.get() };
        g.this = thisref;
        g.groups = groups;
    }

    Ok(g)
}
