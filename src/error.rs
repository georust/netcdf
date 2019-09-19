use super::nc_type;
use super::LOCK;
use netcdf_sys::nc_strerror;

#[derive(Debug)]
pub enum Error {
    Netcdf(nc_type),
    Crate(String),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Error {
        Error::Crate(s.into())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Error {
        Error::Crate(s)
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
            Error::Crate(x) => write!(f, "{}", x),
            Error::Netcdf(x) => {
                let _l = LOCK.lock().unwrap();
                let msg;
                unsafe {
                    let cmsg = nc_strerror(*x);
                    msg = std::ffi::CStr::from_ptr(cmsg);
                }

                write!(f, "netcdf error({}): {}", x, msg.to_string_lossy())
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
