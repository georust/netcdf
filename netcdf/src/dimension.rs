//! Interact with netcdf dimensions
#![allow(clippy::similar_names)]

use std::marker::PhantomData;

use netcdf_sys::*;

use super::error;
use super::utils::with_lock;

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

#[allow(clippy::len_without_is_empty)]
impl<'g> Dimension<'g> {
    /// Get current length of this dimension
    pub fn len(&self) -> usize {
        if let Some(x) = self.len {
            x.get()
        } else {
            let mut len = 0;
            let err = unsafe {
                // Must lock in case other variables adds to the dimension length
                error::checked(with_lock(|| {
                    nc_inq_dimlen(self.id.ncid, self.id.dimid, &mut len)
                }))
            };

            // Should log or handle this somehow...
            err.map(|()| len).unwrap_or(0)
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
            error::checked(with_lock(|| {
                nc_inq_dimname(self.id.ncid, self.id.dimid, name.as_mut_ptr().cast())
            }))
            .unwrap();
        }

        let zeropos = name.iter().position(|&x| x == 0).unwrap_or(name.len());
        name.resize(zeropos, 0);
        String::from_utf8(name).expect("Dimension did not have a valid name")
    }

    /// Grabs the unique identifier for this dimension, which
    /// can be used in `add_variable_from_identifiers`
    pub fn identifier(&self) -> DimensionIdentifier {
        self.id
    }
}

pub(crate) fn from_name_toid(loc: nc_type, name: &str) -> error::Result<Option<nc_type>> {
    let mut dimid = 0;
    let cname = super::utils::short_name_to_bytes(name)?;
    let e = unsafe { with_lock(|| nc_inq_dimid(loc, cname.as_ptr().cast(), &mut dimid)) };
    if e == NC_EBADDIM {
        return Ok(None);
    }
    error::checked(e)?;

    Ok(Some(dimid))
}

pub(crate) fn from_name<'f>(loc: nc_type, name: &str) -> error::Result<Option<Dimension<'f>>> {
    let mut dimid = 0;
    let cname = super::utils::short_name_to_bytes(name)?;
    let e = unsafe { with_lock(|| nc_inq_dimid(loc, cname.as_ptr().cast(), &mut dimid)) };
    if e == NC_EBADDIM {
        return Ok(None);
    }
    error::checked(e)?;

    let mut dimlen = 0;
    unsafe {
        error::checked(with_lock(|| nc_inq_dimlen(loc, dimid, &mut dimlen)))?;
    }
    if dimlen != 0 {
        let mut nunlim = 0;
        unsafe {
            error::checked(with_lock(|| {
                nc_inq_unlimdims(loc, &mut nunlim, std::ptr::null_mut())
            }))?;
        }
        if nunlim != 0 {
            let mut unlimdims = Vec::with_capacity(nunlim.try_into()?);
            unsafe {
                error::checked(with_lock(|| {
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
        error::checked(with_lock(|| {
            nc_inq_dimids(ncid, &mut ndims, std::ptr::null_mut(), <_>::from(false))
        }))?;
    }
    let mut dimids = vec![0; ndims.try_into()?];
    unsafe {
        error::checked(with_lock(|| {
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
            error::checked(with_lock(|| {
                nc_inq_unlimdims(ncid, &mut nunlimdims, std::ptr::null_mut())
            }))?;
        }
        let mut unlimdims = Vec::with_capacity(nunlimdims.try_into()?);
        unsafe {
            error::checked(with_lock(|| {
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
                error::checked(with_lock(|| nc_inq_dimlen(ncid, dimid, &mut dimlen)))?;
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
        error::checked(with_lock(|| nc_inq_varndims(ncid, varid, &mut ndims)))?;
    }
    let mut dimids = vec![0; ndims.try_into()?];
    unsafe {
        error::checked(with_lock(|| {
            nc_inq_vardimid(ncid, varid, dimids.as_mut_ptr())
        }))?;
    }
    let unlimdims = {
        let mut nunlimdims = 0;
        unsafe {
            error::checked(with_lock(|| {
                nc_inq_unlimdims(ncid, &mut nunlimdims, std::ptr::null_mut())
            }))?;
        }
        let mut unlimdims = Vec::with_capacity(nunlimdims.try_into()?);
        unsafe {
            error::checked(with_lock(|| {
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
                error::checked(with_lock(|| nc_inq_dimlen(ncid, dimid, &mut dimlen)))?;
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
    let e = unsafe { with_lock(|| nc_inq_dimid(ncid, cname.as_ptr().cast(), &mut dimid)) };
    if e == NC_EBADDIM {
        return Ok(None);
    }
    error::checked(e)?;

    let mut dimlen = 0;
    unsafe {
        error::checked(with_lock(|| nc_inq_dimlen(ncid, dimid, &mut dimlen))).unwrap();
    }
    if dimlen != 0 {
        // Have to check if this dimension is unlimited
        let mut nunlim = 0;
        unsafe {
            error::checked(with_lock(|| {
                nc_inq_unlimdims(ncid, &mut nunlim, std::ptr::null_mut())
            }))?;
        }
        if nunlim != 0 {
            let mut unlimdims = Vec::with_capacity(nunlim.try_into()?);
            unsafe {
                error::checked(with_lock(|| {
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
        error::checked(with_lock(|| {
            nc_def_dim(ncid, cname.as_ptr().cast(), len, &mut dimid)
        }))?;
    }
    Ok(Dimension {
        len: core::num::NonZeroUsize::new(dimid.try_into()?),
        id: DimensionIdentifier { ncid, dimid },
        _group: PhantomData,
    })
}
