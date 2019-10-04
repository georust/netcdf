use super::nc_type;
use super::LOCK;
use netcdf_sys::nc_strerror;

#[derive(Debug, PartialEq)]
pub enum Error {
    /// Errors from the wrapped netcdf library
    Netcdf(nc_type),
    /// Misc errors
    Str(String),
    /// Length of the request indices is inconsistent
    IndexLen,
    /// Length of the slice indices is insconsistent
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
    /// Slice lenghts are ambigious
    Ambiguous,
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Error {
        Error::Str(s.into())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Error {
        Error::Str(s)
    }
}

impl From<nc_type> for Error {
    fn from(nc: nc_type) -> Error {
        Error::Netcdf(nc)
    }
}

use std::fmt;
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Str(x) => write!(f, "{}", x),
            Error::IndexLen => write!(f, "indices does not match in length with the variable"),
            Error::SliceLen => write!(f, "slices does not match in length with the variable"),
            Error::IndexMismatch => {
                write!(f, "requested index is bigger than the dimension length")
            }
            Error::SliceMismatch => {
                write!(f, "requested slice is bigger than the dimension length")
            }
            Error::ZeroSlice => write!(f, "must request a slice length larger than zero"),
            Error::BufferLen(has, need) => write!(
                f,
                "buffer size mismatch, has size {}, but needs size {}",
                has, need
            ),
            Error::TypeMismatch => write!(f, "netcdf types does not correspond to what is defined"),
            Error::TypeUnknown(t) => write!(f, "netcdf type {} is not known", t),
            Error::AlreadyExists(x) => write!(f, "{} already exists", x),
            Error::NotFound(x) => write!(f, "Could not find {}", x),
            Error::Netcdf(x) => {
                let _l = LOCK.lock().unwrap();
                let msg;
                unsafe {
                    let cmsg = nc_strerror(*x);
                    msg = std::ffi::CStr::from_ptr(cmsg);
                }

                write!(f, "netcdf error({}): {}", x, msg.to_string_lossy())
            }
            Error::Ambiguous => write!(f, "Could not find an appropriate length of the slices"),
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
