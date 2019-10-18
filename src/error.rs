#![allow(clippy::similar_names)]
use super::nc_type;
use super::LOCK;
use netcdf_sys::nc_strerror;
use std::num::TryFromIntError;

#[derive(Debug, PartialEq)]
pub enum Error {
    /// Errors from the wrapped netcdf library
    Netcdf(nc_type),
    /// Misc errors
    Str(String),
    /// Length of the request indices is inconsistent
    IndexLen,
    /// Length of the slice indices is inconsistent
    SliceLen,
    /// Supplied the wrong length of the buffer
    BufferLen(usize, usize),
    /// Some index is greater than expected
    IndexMismatch,
    /// Requested a mismatched total slice
    SliceMismatch,
    /// Requested a zero slice
    ZeroSlice,
    /// Supplied the wrong type of parameter
    TypeMismatch,
    /// Does not know the type (probably library error...)
    TypeUnknown(nc_type),
    /// Variable/dimension already exists
    AlreadyExists(String),
    /// Could not find variable/attribute/etc
    NotFound(String),
    /// Slice lengths are ambiguous
    Ambiguous,
    /// Overflows possible lengths
    Overflow,
    /// Conversion error
    Conversion(TryFromIntError),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::Str(s.into())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Self::Str(s)
    }
}

impl From<nc_type> for Error {
    fn from(nc: nc_type) -> Self {
        Self::Netcdf(nc)
    }
}

impl From<TryFromIntError> for Error {
    fn from(e: TryFromIntError) -> Self {
        Self::Conversion(e)
    }
}

use std::fmt;
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Str(x) => write!(f, "{}", x),
            Self::IndexLen => write!(f, "indices does not match in length with the variable"),
            Self::SliceLen => write!(f, "slices does not match in length with the variable"),
            Self::IndexMismatch => write!(f, "requested index is bigger than the dimension length"),
            Self::SliceMismatch => write!(f, "requested slice is bigger than the dimension length"),
            Self::ZeroSlice => write!(f, "must request a slice length larger than zero"),
            Self::BufferLen(has, need) => write!(
                f,
                "buffer size mismatch, has size {}, but needs size {}",
                has, need
            ),
            Self::TypeMismatch => write!(f, "netcdf types does not correspond to what is defined"),
            Self::TypeUnknown(t) => write!(f, "netcdf type {} is not known", t),
            Self::AlreadyExists(x) => write!(f, "{} already exists", x),
            Self::NotFound(x) => write!(f, "Could not find {}", x),
            Self::Netcdf(x) => {
                let _l = LOCK.lock().unwrap();
                let msg;
                unsafe {
                    let cmsg = nc_strerror(*x);
                    msg = std::ffi::CStr::from_ptr(cmsg);
                }

                write!(f, "netcdf error({}): {}", x, msg.to_string_lossy())
            }
            Self::Ambiguous => write!(f, "Could not find an appropriate length of the slices"),
            Self::Overflow => write!(f, "slice would exceed maximum size of possible buffers"),
            Self::Conversion(e) => e.fmt(f),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn checked(err: nc_type) -> Result<()> {
    if err != netcdf_sys::NC_NOERR {
        return Err(err.into());
    }
    Ok(())
}
