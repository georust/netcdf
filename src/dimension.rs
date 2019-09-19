use super::error;
use super::LOCK;
use netcdf_sys::*;

#[derive(Debug, Clone)]
pub struct Dimension {
    pub(crate) name: String,
    pub(crate) len: usize,
    pub(crate) id: nc_type,
}

impl Dimension {
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn new(grpid: nc_type, name: &str, len: usize) -> error::Result<Dimension> {
        use std::ffi::CString;

        let mut dimid = 0;
        let cname = CString::new(name).unwrap();
        let err;

        unsafe {
            let _l = LOCK.lock().unwrap();
            err = nc_def_dim(grpid, cname.as_ptr(), len, &mut dimid);
        }
        if err != NC_NOERR {
            return Err(err.into());
        }

        Ok(Dimension {
            name: name.into(),
            len,
            id: dimid,
        })
    }
}
