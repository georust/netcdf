//! Errors that can appear when interacting with netcdf files.
//! This module contains conversion traits and the result type
//! used in this crate.
#![allow(clippy::similar_names)]

use std::num::TryFromIntError;

use netcdf_sys::nc_strerror;

use super::nc_type;

/// Various error types that can occur in this crate
#[derive(Debug)]
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
    BufferLen {
        /// Wanted size of the buffer
        wanted: usize,
        /// Actual size of the buffer
        actual: usize,
    },
    /// Supplied the wrong length of dimension
    DimensionMismatch {
        /// Wanted size of the dimension
        wanted: usize,
        /// Actual size of the dimension
        actual: usize,
    },
    /// Some index is greater than expected
    IndexMismatch,
    /// Requested a mismatched total slice
    SliceMismatch,
    /// Requested a zero slice
    ZeroSlice,
    /// Zero stride or matched with length != 1
    Stride,
    /// Supplied the wrong type of parameter
    TypeMismatch,
    /// Does not know the type (probably library error...)
    TypeUnknown(nc_type),
    /// Variable/dimension already exists
    AlreadyExists,
    /// Could not find variable/attribute/etc
    NotFound(String),
    /// Slice lengths are ambiguous
    Ambiguous,
    /// Overflows possible lengths
    Overflow,
    /// Conversion error
    Conversion(TryFromIntError),
    /// Identifier belongs to another dataset
    WrongDataset,
    /// Name is not valid utf-8
    Utf8Conversion(std::string::FromUtf8Error),
    /// String contains NULL characters
    NulError(std::ffi::NulError),
}

impl Error {
    /// Was the error due to ambiguity of the
    /// indices or lengths?
    #[deprecated(note = "This item has a typo, use is_ambiguous instead")]
    pub fn is_ambigous(&self) -> bool {
        matches!(self, Self::Ambiguous)
    }
    /// Was the error due to ambiguity of the
    /// indices or lengths?
    pub fn is_ambiguous(&self) -> bool {
        matches!(self, Self::Ambiguous)
    }
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
        if nc == netcdf_sys::NC_EEXIST
            || nc == netcdf_sys::NC_EATTEXISTS
            || nc == netcdf_sys::NC_ENAMEINUSE
        {
            Self::AlreadyExists
        } else {
            Self::Netcdf(nc)
        }
    }
}

impl From<TryFromIntError> for Error {
    fn from(e: TryFromIntError) -> Self {
        Self::Conversion(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::Utf8Conversion(e)
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(e: std::ffi::NulError) -> Self {
        Self::NulError(e)
    }
}

impl From<std::convert::Infallible> for Error {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!("Infallible error can never be constructed")
    }
}

use std::fmt;
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Str(x) => x.fmt(f),
            Self::IndexLen => write!(f, "indices does not match in length with the variable"),
            Self::SliceLen => write!(f, "slices does not match in length with the variable"),
            Self::IndexMismatch => write!(f, "requested index is bigger than the dimension length"),
            Self::SliceMismatch => write!(f, "requested slice is bigger than the dimension length"),
            Self::DimensionMismatch { wanted, actual } => write!(
                f,
                "requested dimension ({actual}) is bigger than the dimension length ({wanted})"
            ),
            Self::ZeroSlice => write!(f, "must request a slice length larger than zero"),
            Self::Stride => write!(f, "invalid strides"),
            Self::BufferLen { wanted, actual } => write!(
                f,
                "buffer size mismatch, has size {actual}, but needs size {wanted}",
            ),
            Self::TypeMismatch => write!(f, "netcdf types does not correspond to what is defined"),
            Self::TypeUnknown(t) => write!(f, "netcdf type {t} is not known"),
            Self::AlreadyExists => write!(f, "variable/group/dimension already exists"),
            Self::NotFound(x) => write!(f, "could not find {x}"),
            Self::Netcdf(x) => {
                let msg;
                unsafe {
                    // Threadsafe
                    let cmsg = nc_strerror(*x);
                    msg = std::ffi::CStr::from_ptr(cmsg);
                }

                write!(f, "netcdf error({}): {}", x, msg.to_string_lossy())
            }
            Self::Ambiguous => write!(f, "could not find an appropriate length of the slices"),
            Self::Overflow => write!(f, "slice would exceed maximum size of possible buffers"),
            Self::Conversion(e) => e.fmt(f),
            Self::WrongDataset => write!(f, "this identifier does not belong in this dataset"),
            Self::Utf8Conversion(e) => e.fmt(f),
            Self::NulError(e) => write!(f, "string value contains null bytes {e}"),
        }
    }
}

/// Result type used in this crate
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub(crate) fn checked(err: nc_type) -> Result<()> {
    if err != netcdf_sys::NC_NOERR {
        return Err(err.into());
    }
    Ok(())
}
