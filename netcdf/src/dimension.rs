//! Interact with netcdf dimensions
#![allow(clippy::similar_names)]

use std::marker::PhantomData;

use netcdf_sys::*;

use super::error;

mod sealed {
    pub trait Sealed {}
}

/// Types which can be used to distinguish dimensions
/// in a netCDF file. This can be `&str` (the normal use case)
/// or a special identifier when using nested groups.
///
/// This trait is not expected to be implemented elsewhere and is therefore sealed.
///
/// # Examples
/// (helper function to show consumption of type)
/// ```rust,no_run
/// # use netcdf::AsNcDimensions;
/// fn take(d: impl AsNcDimensions) {}
/// ```
/// Normally one uses the name of the dimension to specify the dimension
/// ```rust,no_run
/// # use netcdf::AsNcDimensions;
/// # fn take(d: impl AsNcDimensions) {}
/// take(()); // scalar
/// take("x"); // single dimension
/// take(["x", "y"]); // multiple dimensions
/// ```
/// When working with dimensions across groups, it might be necessary
/// to use dimension identifiers to get the correct group dimension
/// ```rust,no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use netcdf::AsNcDimensions;
/// # fn take(d: impl AsNcDimensions) {}
/// let file = netcdf::open("test.nc")?;
/// let dim = file.dimension("x").expect("File does not contain dimension");
/// let dimid = dim.identifier();
///
/// take(dimid); // from a dimension identifier
/// take([dimid, dimid]); // from multiple identifiers
/// # Ok(()) }
/// ```
pub trait AsNcDimensions: sealed::Sealed {
    /// Convert from a slice of [`&str`]/[`DimensionIdentifier`] to concrete dimensions
    /// which are guaranteed to exist in this file
    fn get_dimensions<'g>(&self, ncid: nc_type) -> error::Result<Vec<Dimension<'g>>>;
}

impl sealed::Sealed for &[&str] {}
impl AsNcDimensions for &[&str] {
    fn get_dimensions<'g>(&self, ncid: nc_type) -> error::Result<Vec<Dimension<'g>>> {
        self.iter()
            .map(|&name| match dimension_from_name(ncid, name) {
                Ok(Some(x)) => Ok(x),
                Ok(None) => Err(format!("Dimension {name} not found").into()),
                Err(e) => Err(e),
            })
            .collect()
    }
}
impl<const N: usize> sealed::Sealed for [&str; N] {}
impl<const N: usize> AsNcDimensions for [&str; N] {
    fn get_dimensions<'g>(&self, ncid: nc_type) -> error::Result<Vec<Dimension<'g>>> {
        self.as_slice().get_dimensions(ncid)
    }
}
impl<const N: usize> sealed::Sealed for &[&str; N] {}
impl<const N: usize> AsNcDimensions for &[&str; N] {
    fn get_dimensions<'g>(&self, ncid: nc_type) -> error::Result<Vec<Dimension<'g>>> {
        self.as_slice().get_dimensions(ncid)
    }
}

impl sealed::Sealed for &[DimensionIdentifier] {}
impl AsNcDimensions for &[DimensionIdentifier] {
    fn get_dimensions<'g>(&self, ncid: nc_type) -> error::Result<Vec<Dimension<'g>>> {
        self.iter()
            .map(|dimid| match dimension_from_identifier(ncid, *dimid) {
                Ok(Some(x)) => Ok(x),
                Ok(None) => Err("Dimension id does not exist".into()),
                Err(e) => Err(e),
            })
            .collect()
    }
}
impl<const N: usize> sealed::Sealed for [DimensionIdentifier; N] {}
impl<const N: usize> AsNcDimensions for [DimensionIdentifier; N] {
    fn get_dimensions<'g>(&self, ncid: nc_type) -> error::Result<Vec<Dimension<'g>>> {
        self.as_slice().get_dimensions(ncid)
    }
}
impl<const N: usize> sealed::Sealed for &[DimensionIdentifier; N] {}
impl<const N: usize> AsNcDimensions for &[DimensionIdentifier; N] {
    fn get_dimensions<'g>(&self, ncid: nc_type) -> error::Result<Vec<Dimension<'g>>> {
        self.as_slice().get_dimensions(ncid)
    }
}
impl sealed::Sealed for () {}
impl AsNcDimensions for () {
    fn get_dimensions<'g>(&self, _ncid: nc_type) -> error::Result<Vec<Dimension<'g>>> {
        Ok(Vec::new())
    }
}
impl sealed::Sealed for &str {}
impl AsNcDimensions for &str {
    fn get_dimensions<'g>(&self, ncid: nc_type) -> error::Result<Vec<Dimension<'g>>> {
        ([*self]).get_dimensions(ncid)
    }
}
impl sealed::Sealed for DimensionIdentifier {}
impl AsNcDimensions for DimensionIdentifier {
    fn get_dimensions<'g>(&self, ncid: nc_type) -> error::Result<Vec<Dimension<'g>>> {
        ([*self]).get_dimensions(ncid)
    }
}

/// Represents a netcdf dimension
#[derive(Debug, Clone)]
pub struct Dimension<'g> {
    /// None when unlimited (size = 0)
    pub(crate) len: Option<core::num::NonZeroUsize>,
    pub(crate) id: DimensionIdentifier,
    pub(crate) _group: PhantomData<&'g nc_type>,
}

/// Unique identifier for a dimension in a file. Used when
/// names can not be used directly, for example when dealing
/// with nested groups
#[derive(Debug, Copy, Clone)]
pub struct DimensionIdentifier {
    pub(crate) ncid: nc_type,
    pub(crate) dimid: nc_type,
}

impl DimensionIdentifier {
    // Internal netcdf detail, the top 16 bits gives the corresponding
    // file handle. This to ensure dimensions are not added from another
    // file which is unrelated to self
    pub(crate) fn belongs_to(&self, ncid: nc_type) -> bool {
        self.ncid >> 16 == ncid >> 16
    }
}

#[allow(clippy::len_without_is_empty)]
impl<'g> Dimension<'g> {
    /// Get current length of this dimension
    ///
    /// ## Note
    /// A dimension can be unlimited (growable) and changes size
    /// if putting values to a variable which uses this
    /// dimension
    pub fn len(&self) -> usize {
        if let Some(x) = self.len {
            x.get()
        } else {
            let mut len = 0;
            let err = unsafe {
                // Must lock in case other variables adds to the dimension length
                error::checked(super::with_lock(|| {
                    nc_inq_dimlen(self.id.ncid, self.id.dimid, &mut len)
                }))
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
    pub fn name(&self) -> String {
        let mut name = vec![0_u8; NC_MAX_NAME as usize + 1];
        unsafe {
            error::checked(super::with_lock(|| {
                nc_inq_dimname(self.id.ncid, self.id.dimid, name.as_mut_ptr().cast())
            }))
            .unwrap();
        }

        let zeropos = name.iter().position(|&x| x == 0).unwrap_or(name.len());
        name.resize(zeropos, 0);
        String::from_utf8(name).expect("Dimension did not have a valid name")
    }

    /// Grabs the unique identifier for this dimension, which
    /// can be used in [`add_variable`](crate::FileMut::add_variable).
    ///
    /// This is useful when working with nested groups and need
    /// to distinguish between names at different levels.
    pub fn identifier(&self) -> DimensionIdentifier {
        self.id
    }
}

pub(crate) fn from_name_toid(loc: nc_type, name: &str) -> error::Result<Option<nc_type>> {
    let mut dimid = 0;
    let cname = super::utils::short_name_to_bytes(name)?;
    let e = unsafe { super::with_lock(|| nc_inq_dimid(loc, cname.as_ptr().cast(), &mut dimid)) };
    if e == NC_EBADDIM {
        return Ok(None);
    }
    error::checked(e)?;

    Ok(Some(dimid))
}

pub(crate) fn from_name<'f>(loc: nc_type, name: &str) -> error::Result<Option<Dimension<'f>>> {
    let mut dimid = 0;
    let cname = super::utils::short_name_to_bytes(name)?;
    let e = unsafe { super::with_lock(|| nc_inq_dimid(loc, cname.as_ptr().cast(), &mut dimid)) };
    if e == NC_EBADDIM {
        return Ok(None);
    }
    error::checked(e)?;

    let mut dimlen = 0;
    unsafe {
        error::checked(super::with_lock(|| nc_inq_dimlen(loc, dimid, &mut dimlen)))?;
    }
    if dimlen != 0 {
        let mut nunlim = 0;
        unsafe {
            error::checked(super::with_lock(|| {
                nc_inq_unlimdims(loc, &mut nunlim, std::ptr::null_mut())
            }))?;
        }
        if nunlim != 0 {
            let mut unlimdims = Vec::with_capacity(nunlim.try_into()?);
            unsafe {
                error::checked(super::with_lock(|| {
                    nc_inq_unlimdims(loc, std::ptr::null_mut(), unlimdims.as_mut_ptr())
                }))?;
            }
            unsafe { unlimdims.set_len(nunlim.try_into()?) }
            if unlimdims.contains(&dimid) {
                dimlen = 0;
            }
        }
    }

    Ok(Some(Dimension {
        len: core::num::NonZeroUsize::new(dimlen),
        id: DimensionIdentifier { ncid: loc, dimid },
        _group: PhantomData,
    }))
}

pub(crate) fn dimensions_from_location<'g>(
    ncid: nc_type,
) -> error::Result<impl Iterator<Item = error::Result<Dimension<'g>>>> {
    let mut ndims = 0;
    unsafe {
        error::checked(super::with_lock(|| {
            nc_inq_dimids(ncid, &mut ndims, std::ptr::null_mut(), <_>::from(false))
        }))?;
    }
    let mut dimids = vec![0; ndims.try_into()?];
    unsafe {
        error::checked(super::with_lock(|| {
            nc_inq_dimids(
                ncid,
                std::ptr::null_mut(),
                dimids.as_mut_ptr(),
                <_>::from(false),
            )
        }))?;
    }
    let unlimdims = {
        let mut nunlimdims = 0;
        unsafe {
            error::checked(super::with_lock(|| {
                nc_inq_unlimdims(ncid, &mut nunlimdims, std::ptr::null_mut())
            }))?;
        }
        let mut unlimdims = Vec::with_capacity(nunlimdims.try_into()?);
        unsafe {
            error::checked(super::with_lock(|| {
                nc_inq_unlimdims(ncid, std::ptr::null_mut(), unlimdims.as_mut_ptr())
            }))?;
        }
        unsafe {
            unlimdims.set_len(nunlimdims.try_into()?);
        }
        unlimdims
    };
    Ok(dimids.into_iter().map(move |dimid| {
        let mut dimlen = 0;
        if !unlimdims.contains(&dimid) {
            unsafe {
                error::checked(super::with_lock(|| nc_inq_dimlen(ncid, dimid, &mut dimlen)))?;
            }
        }
        Ok(Dimension {
            len: core::num::NonZeroUsize::new(dimlen),
            id: DimensionIdentifier { ncid, dimid },
            _group: PhantomData,
        })
    }))
}

pub(crate) fn dimensions_from_variable<'g>(
    ncid: nc_type,
    varid: nc_type,
) -> error::Result<impl Iterator<Item = error::Result<Dimension<'g>>>> {
    let mut ndims = 0;
    unsafe {
        error::checked(super::with_lock(|| {
            nc_inq_varndims(ncid, varid, &mut ndims)
        }))?;
    }
    let mut dimids = vec![0; ndims.try_into()?];
    unsafe {
        error::checked(super::with_lock(|| {
            nc_inq_vardimid(ncid, varid, dimids.as_mut_ptr())
        }))?;
    }
    let unlimdims = {
        let mut nunlimdims = 0;
        unsafe {
            error::checked(super::with_lock(|| {
                nc_inq_unlimdims(ncid, &mut nunlimdims, std::ptr::null_mut())
            }))?;
        }
        let mut unlimdims = Vec::with_capacity(nunlimdims.try_into()?);
        unsafe {
            error::checked(super::with_lock(|| {
                nc_inq_unlimdims(ncid, std::ptr::null_mut(), unlimdims.as_mut_ptr())
            }))?;
        }
        unsafe {
            unlimdims.set_len(nunlimdims.try_into()?);
        }
        unlimdims
    };

    Ok(dimids.into_iter().map(move |dimid| {
        let mut dimlen = 0;
        if !unlimdims.contains(&dimid) {
            unsafe {
                error::checked(super::with_lock(|| nc_inq_dimlen(ncid, dimid, &mut dimlen)))?;
            }
        }
        Ok(Dimension {
            len: core::num::NonZeroUsize::new(dimlen),
            id: DimensionIdentifier { ncid, dimid },
            _group: PhantomData,
        })
    }))
}

pub(crate) fn dimension_from_name<'f>(
    ncid: nc_type,
    name: &str,
) -> error::Result<Option<Dimension<'f>>> {
    let cname = super::utils::short_name_to_bytes(name)?;
    let mut dimid = 0;
    let e = unsafe { super::with_lock(|| nc_inq_dimid(ncid, cname.as_ptr().cast(), &mut dimid)) };
    if e == NC_EBADDIM {
        return Ok(None);
    }
    error::checked(e)?;

    let mut dimlen = 0;
    unsafe {
        error::checked(super::with_lock(|| nc_inq_dimlen(ncid, dimid, &mut dimlen))).unwrap();
    }
    if dimlen != 0 {
        // Have to check if this dimension is unlimited
        let mut nunlim = 0;
        unsafe {
            error::checked(super::with_lock(|| {
                nc_inq_unlimdims(ncid, &mut nunlim, std::ptr::null_mut())
            }))?;
        }
        if nunlim != 0 {
            let mut unlimdims = Vec::with_capacity(nunlim.try_into()?);
            unsafe {
                error::checked(super::with_lock(|| {
                    nc_inq_unlimdims(ncid, std::ptr::null_mut(), unlimdims.as_mut_ptr())
                }))?;
            }
            unsafe { unlimdims.set_len(nunlim.try_into()?) }
            if unlimdims.contains(&dimid) {
                dimlen = 0;
            }
        }
    }
    Ok(Some(Dimension {
        len: core::num::NonZeroUsize::new(dimlen),
        id: super::dimension::DimensionIdentifier { ncid, dimid },
        _group: PhantomData,
    }))
}

pub(crate) fn dimension_from_identifier<'f>(
    ncid: nc_type,
    dimid: DimensionIdentifier,
) -> error::Result<Option<Dimension<'f>>> {
    if !dimid.belongs_to(ncid) {
        return Err(error::Error::WrongDataset);
    }
    let dimid = dimid.dimid;

    let mut dimlen = 0;
    unsafe {
        error::checked(super::with_lock(|| nc_inq_dimlen(ncid, dimid, &mut dimlen))).unwrap();
    }
    if dimlen != 0 {
        // Have to check if this dimension is unlimited
        let mut nunlim = 0;
        unsafe {
            error::checked(super::with_lock(|| {
                nc_inq_unlimdims(ncid, &mut nunlim, std::ptr::null_mut())
            }))?;
        }
        if nunlim != 0 {
            let mut unlimdims = Vec::with_capacity(nunlim.try_into()?);
            unsafe {
                error::checked(super::with_lock(|| {
                    nc_inq_unlimdims(ncid, std::ptr::null_mut(), unlimdims.as_mut_ptr())
                }))?;
            }
            unsafe { unlimdims.set_len(nunlim.try_into()?) }
            if unlimdims.contains(&dimid) {
                dimlen = 0;
            }
        }
    }
    Ok(Some(Dimension {
        len: core::num::NonZeroUsize::new(dimlen),
        id: super::dimension::DimensionIdentifier { ncid, dimid },
        _group: PhantomData,
    }))
}

pub(crate) fn add_dimension_at<'f>(
    ncid: nc_type,
    name: &str,
    len: usize,
) -> error::Result<Dimension<'f>> {
    let cname = super::utils::short_name_to_bytes(name)?;
    let mut dimid = 0;
    unsafe {
        error::checked(super::with_lock(|| {
            nc_def_dim(ncid, cname.as_ptr().cast(), len, &mut dimid)
        }))?;
    }
    Ok(Dimension {
        len: core::num::NonZeroUsize::new(dimid.try_into()?),
        id: DimensionIdentifier { ncid, dimid },
        _group: PhantomData,
    })
}
