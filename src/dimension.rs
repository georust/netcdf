//! Interact with netcdf dimensions

#![allow(clippy::similar_names)]
use super::error;
use super::LOCK;
use netcdf_sys::*;

/// Represents a netcdf dimension
#[derive(Debug, Clone)]
pub struct Dimension {
    pub(crate) name: String,
    /// None when unlimited (size = 0)
    pub(crate) len: Option<core::num::NonZeroUsize>,
    pub(crate) id: nc_type,
    pub(crate) ncid: nc_type,
}

/// Unique identifier for a dimensions in a file. Used when
/// names can not be used directly
#[derive(Debug, Copy, Clone)]
pub struct Identifier {
    pub(crate) ncid: nc_type,
    pub(crate) identifier: nc_type,
}

#[allow(clippy::len_without_is_empty)]
impl Dimension {
    /// Get current length of the dimensions, which is
    /// the product of all dimensions in the variable
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

    /// Checks whether the dimension is growable
    pub fn is_unlimited(&self) -> bool {
        self.len.is_none()
    }

    /// Gets the name of the dimension
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Grabs a unique identifier for this dimension
    pub fn identifier(&self) -> Identifier {
        Identifier {
            ncid: self.ncid,
            identifier: self.id,
        }
    }

    pub(crate) fn new(grpid: nc_type, name: String, len: usize) -> error::Result<Self> {
        use std::ffi::CString;

        let mut dimid = 0;
        let cname = CString::new(name.as_str()).unwrap();

        unsafe {
            let _l = LOCK.lock().unwrap();
            error::checked(nc_def_dim(grpid, cname.as_ptr(), len, &mut dimid))?;
        }

        Ok(Self {
            name,
            len: core::num::NonZeroUsize::new(len),
            id: dimid,
            ncid: grpid,
        })
    }
}
