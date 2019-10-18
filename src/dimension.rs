#![allow(clippy::similar_names)]
use super::error;
use super::LOCK;
use netcdf_sys::*;

#[derive(Debug, Clone)]
pub struct Dimension {
    pub(crate) name: String,
    /// None when unlimited (size = 0)
    pub(crate) len: Option<core::num::NonZeroUsize>,
    pub(crate) id: nc_type,
    pub(crate) ncid: nc_type,
}

#[derive(Debug, Copy, Clone)]
pub struct Identifier {
    pub(crate) identifier: nc_type,
}

#[allow(clippy::len_without_is_empty)]
impl Dimension {
    pub fn len(&self) -> usize {
        if let Some(x) = self.len {
            x.get()
        } else {
            let mut len = 0;
            let err = unsafe {
                let _l = LOCK.lock().unwrap();
                error::checked(nc_inq_dimlen(self.ncid, self.id, &mut len))
            };

            // Should log or handle this somehow...
            err.map(|_| len).unwrap_or(0)
        }
    }

    pub fn is_unlimited(&self) -> bool {
        self.len.is_none()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn identifier(&self) -> Identifier {
        Identifier {
            identifier: self.id,
        }
    }

    pub(crate) fn new(grpid: nc_type, name: &str, len: usize) -> error::Result<Self> {
        use std::ffi::CString;

        let mut dimid = 0;
        let cname = CString::new(name).unwrap();

        unsafe {
            let _l = LOCK.lock().unwrap();
            error::checked(nc_def_dim(grpid, cname.as_ptr(), len, &mut dimid))?;
        }

        Ok(Self {
            name: name.into(),
            len: core::num::NonZeroUsize::new(len),
            id: dimid,
            ncid: grpid,
        })
    }
}
