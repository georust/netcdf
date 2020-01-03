//! Open, create, and append netcdf files

#![allow(clippy::similar_names)]
use super::attribute::{AttrValue, Attribute};
use super::dimension::{Dimension, Identifier};
use super::error;
use super::group::{Group, GroupMut};
use super::variable::{Numeric, Variable, VariableMut};
use super::LOCK;
use netcdf_sys::*;
use std::ffi::CString;
use std::marker::PhantomData;
use std::path;

#[derive(Debug)]
pub(crate) struct File {
    ncid: nc_type,
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

impl File {
    /// Open a netCDF file in read only mode.
    ///
    /// Consider using [`netcdf::open`] instead to open with
    /// a generic `Path` object, and ensure read-only on
    /// the `File`
    pub(crate) fn open(path: &path::Path) -> error::Result<ReadOnlyFile> {
        let f = CString::new(path.to_str().unwrap()).unwrap();
        let mut ncid: nc_type = 0;
        unsafe {
            let _l = LOCK.lock().unwrap();
            error::checked(nc_open(f.as_ptr(), NC_NOWRITE, &mut ncid))?;
        }
        Ok(ReadOnlyFile(Self { ncid }))
    }

    #[allow(clippy::doc_markdown)]
    /// Open a netCDF file in append mode (read/write).
    /// The file must already exist.
    pub(crate) fn append(path: &path::Path) -> error::Result<MutableFile> {
        let f = CString::new(path.to_str().unwrap()).unwrap();
        let mut ncid: nc_type = -1;
        unsafe {
            let _g = LOCK.lock().unwrap();
            error::checked(nc_open(f.as_ptr(), NC_WRITE, &mut ncid))?;
        }

        Ok(MutableFile(ReadOnlyFile(Self { ncid })))
    }
    #[allow(clippy::doc_markdown)]
    /// Open a netCDF file in creation mode.
    ///
    /// Will overwrite existing file if any
    pub(crate) fn create(path: &path::Path) -> error::Result<MutableFile> {
        let f = CString::new(path.to_str().unwrap()).unwrap();
        let mut ncid: nc_type = -1;
        unsafe {
            let _g = LOCK.lock().unwrap();
            error::checked(nc_create(f.as_ptr(), NC_NETCDF4 | NC_CLOBBER, &mut ncid))?;
        }

        Ok(MutableFile(ReadOnlyFile(Self { ncid })))
    }

    #[cfg(feature = "memory")]
    pub(crate) fn open_from_memory<'buffer>(
        name: Option<&str>,
        mem: &'buffer [u8],
    ) -> error::Result<MemFile<'buffer>> {
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

        Ok(MemFile(ReadOnlyFile(Self { ncid }), PhantomData))
    }
}

#[derive(Debug)]
pub struct ReadOnlyFile(File);

impl ReadOnlyFile {
    /// path used ot open/create the file
    ///
    /// #Errors
    ///
    /// Netcdf layer could fail, and the resulting path
    /// could contain an invalid UTF8 sequence
    pub fn path(&self) -> error::Result<String> {
        let name = {
            let mut pathlen = 0;
            unsafe {
                error::checked(nc_inq_path(self.0.ncid, &mut pathlen, std::ptr::null_mut()))?;
            }
            let mut name = vec![0_u8; pathlen as _];
            unsafe {
                error::checked(nc_inq_path(
                    self.0.ncid,
                    std::ptr::null_mut(),
                    name.as_mut_ptr() as *mut _,
                ))?;
            }
            name
        };

        String::from_utf8(name).map_err(|e| e.into())
    }

    /// Main entrypoint for interacting with the netcdf file.
    pub fn root<'f>(&'f self) -> Group<'f> {
        Group {
            ncid: self.ncid(),
            _file: PhantomData,
        }
    }

    pub fn variable<'g>(&'g self, name: &str) -> error::Result<Option<Variable<'g, 'g>>> {
        let cname = std::ffi::CString::new(name).unwrap();
        let mut varid = 0;
        let e = unsafe { nc_inq_varid(self.0.ncid, cname.as_ptr(), &mut varid) };
        if e == NC_ENOTFOUND {
            return Ok(None);
        } else {
            error::checked(e)?;
        }
        let mut xtype = 0;
        let mut ndims = 0;
        unsafe {
            error::checked(nc_inq_var(
                self.0.ncid,
                varid,
                std::ptr::null_mut(),
                &mut xtype,
                &mut ndims,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ))?;
        }
        let mut dimids = vec![0; ndims as _];
        unsafe {
            error::checked(nc_inq_vardimid(self.0.ncid, varid, dimids.as_mut_ptr()))?;
        }
        let dimensions = dimids
            .into_iter()
            .map(|id| {
                let mut len = 0;
                unsafe { error::checked(nc_inq_dimlen(self.0.ncid, id, &mut len))? }
                Ok(Dimension {
                    len: core::num::NonZeroUsize::new(len),
                    id: Identifier {
                        ncid: self.0.ncid,
                        dimid: id,
                    },
                    _group: PhantomData,
                })
            })
            .collect::<error::Result<Vec<_>>>()?;

        Ok(Some(Variable {
            dimensions,
            ncid: self.0.ncid,
            varid,
            vartype: xtype,
            _group: PhantomData,
        }))
    }
    pub fn group(&self, name: &str) -> Option<Group> {
        let cname = std::ffi::CString::new(name).unwrap();
        let mut grpid = 0;
        unsafe {
            error::checked(nc_inq_grp_ncid(self.ncid(), cname.as_ptr(), &mut grpid)).unwrap();
        }
        Some(Group {
            ncid: grpid,
            _file: PhantomData,
        })
    }
    pub fn groups<'g>(&'g self) -> impl Iterator<Item = Group<'g>> {
        (0..).into_iter().map(|_| todo!())
    }
    pub fn dimension(&self, name: &str) -> Option<Dimension> {
        let cname = std::ffi::CString::new(name).unwrap();
        let mut dimid = 0;
        let e = unsafe { nc_inq_dimid(self.0.ncid, cname.as_ptr(), &mut dimid) };
        if e == NC_ENOTFOUND {
            return None;
        } else {
            error::checked(e).unwrap();
        }
        let mut dimlen = 0;
        unsafe {
            error::checked(nc_inq_dimlen(self.0.ncid, dimid, &mut dimlen)).unwrap();
        }
        Some(Dimension {
            len: core::num::NonZeroUsize::new(dimlen),
            id: Identifier {
                ncid: self.0.ncid,
                dimid,
            },
            _group: PhantomData,
        })
    }
    pub fn dimensions<'g>(&'g self) -> impl Iterator<Item = Dimension<'g>> {
        let mut ndims = 0;
        unsafe {
            error::checked(nc_inq_dimids(
                self.ncid(),
                &mut ndims,
                std::ptr::null_mut(),
                false as _,
            ))
            .unwrap();
        }
        let mut dimids = vec![0; ndims as _];
        unsafe {
            error::checked(nc_inq_dimids(
                self.ncid(),
                std::ptr::null_mut(),
                dimids.as_mut_ptr(),
                false as _,
            ))
            .unwrap();
        }
        dimids.into_iter().map(move |dimid| {
            let mut dimlen = 0;
            unsafe {
                error::checked(nc_inq_dimlen(self.ncid(), dimid, &mut dimlen)).unwrap();
            }
            Dimension {
                len: core::num::NonZeroUsize::new(dimlen),
                id: Identifier {
                    ncid: self.ncid(),
                    dimid,
                },
                _group: PhantomData,
            }
        })
    }
    pub fn attribute<'f>(&'f self, name: &str) -> error::Result<Option<Attribute<'f>>> {
        if name.len() > NC_MAX_NAME as _ {
            return Err("Name too long".into());
        }
        let mut cname = [0_u8; NC_MAX_NAME as usize + 1];
        cname[..name.len()].copy_from_slice(name.as_bytes());
        let e = unsafe {
            let mut attid = 0;
            nc_inq_attid(
                self.0.ncid,
                NC_GLOBAL,
                cname.as_ptr() as *const _,
                &mut attid,
            )
        };
        if e == NC_ENOTATT {
            Ok(None)
        } else {
            error::checked(e)?;
            Ok(Some(Attribute {
                name: cname,
                ncid: self.0.ncid,
                varid: NC_GLOBAL,
                _marker: PhantomData,
            }))
        }
    }
    pub fn attributes<'f>(
        &'f self,
    ) -> error::Result<impl Iterator<Item = error::Result<Attribute<'f>>>> {
        let _l = super::LOCK.lock().unwrap();
        crate::attribute::AttributeIterator::new(self.0.ncid, None)
    }
    pub fn variables<'f>(
        &'f self,
    ) -> error::Result<impl Iterator<Item = error::Result<Variable<'f, 'f>>>> {
        let mut nvars = 0;
        unsafe {
            error::checked(nc_inq_varids(self.ncid(), &mut nvars, std::ptr::null_mut()))?;
        }
        let mut varids = vec![0; nvars as _];
        unsafe {
            error::checked(nc_inq_varids(
                self.ncid(),
                std::ptr::null_mut(),
                varids.as_mut_ptr(),
            ))?;
        }
        let ncid = self.ncid();
        Ok(varids.into_iter().map(move |varid| {
            let mut ndims = 0;
            let mut xtype = 0;
            unsafe {
                error::checked(nc_inq_var(
                    ncid,
                    varid,
                    std::ptr::null_mut(),
                    &mut xtype,
                    &mut ndims,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ))?;
            }
            let mut dimids = vec![0; ndims as _];
            unsafe {
                error::checked(nc_inq_vardimid(ncid, varid, dimids.as_mut_ptr()))?;
            }

            let dimensions = dimids
                .into_iter()
                .map(|dimid| {
                    let mut dimlen = 0;
                    unsafe {
                        error::checked(nc_inq_dimlen(ncid, dimid, &mut dimlen))?;
                    }
                    Ok(Dimension {
                        len: core::num::NonZeroUsize::new(dimlen),
                        id: Identifier { ncid: ncid, dimid },
                        _group: PhantomData,
                    })
                })
                .collect::<error::Result<Vec<_>>>()?;

            Ok(Variable {
                ncid: self.ncid(),
                varid,
                dimensions,
                vartype: xtype,
                _group: PhantomData,
            })
        }))
    }
    fn ncid(&self) -> nc_type {
        self.0.ncid
    }
}

/// Mutable access to file
#[derive(Debug)]
pub struct MutableFile(ReadOnlyFile);

impl std::ops::Deref for MutableFile {
    type Target = ReadOnlyFile;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MutableFile {
    /// Mutable access to the root group
    pub fn root_mut<'f>(&'f mut self) -> GroupMut<'f> {
        GroupMut(self.root(), PhantomData)
    }

    pub fn add_variable<'f, T>(
        &'f mut self,
        name: &str,
        dims: &[&str],
    ) -> error::Result<VariableMut<'f, 'f>>
    where
        T: Numeric,
    {
        VariableMut::add_from_str((self.0).0.ncid, T::NCTYPE, name, dims)
    }

    pub fn add_dimension<'g>(&'g mut self, name: &str, len: usize) -> error::Result<Dimension<'g>> {
        let cname = std::ffi::CString::new(name).unwrap();
        let mut dimid = 0;
        unsafe {
            error::checked(nc_def_dim(self.ncid(), cname.as_ptr(), len, &mut dimid))?;
        }
        Ok(Dimension {
            len: core::num::NonZeroUsize::new(dimid as _),
            id: Identifier {
                ncid: self.ncid(),
                dimid,
            },
            _group: PhantomData,
        })
    }
    pub fn add_unlimited_dimension(&mut self, name: &str) -> error::Result<Dimension> {
        self.add_dimension(name, 0)
    }
    pub fn group_mut<'f>(&'f mut self, name: &str) -> Option<GroupMut<'f>> {
        self.group(name).map(|g| GroupMut(g, PhantomData))
    }
    pub fn add_variable_from_identifiers<T>(
        &mut self,
        name: &str,
        dims: &[super::dimension::Identifier],
    ) -> error::Result<VariableMut>
    where
        T: Numeric,
    {
        let odims = dims;
        let dims = dims.iter().map(|x| x.dimid).collect::<Vec<_>>();
        let cname = std::ffi::CString::new(name).unwrap();
        let xtype = T::NCTYPE;

        let mut varid = 0;
        unsafe {
            error::checked(nc_def_var(
                (self.0).0.ncid,
                cname.as_ptr(),
                xtype,
                dims.len() as _,
                dims.as_ptr(),
                &mut varid,
            ))?;
        }
        let dimensions = odims
            .into_iter()
            .map(|id| {
                let mut dimlen = 0;
                unsafe {
                    error::checked(nc_inq_dimlen(id.ncid, id.dimid, &mut dimlen))?;
                }
                Ok(Dimension {
                    len: core::num::NonZeroUsize::new(dimlen),
                    id: id.clone(),
                    _group: PhantomData,
                })
            })
            .collect::<error::Result<Vec<_>>>()?;

        Ok(VariableMut(
            Variable {
                ncid: self.ncid(),
                dimensions,
                varid,
                vartype: xtype,
                _group: PhantomData,
            },
            PhantomData,
        ))
    }
    pub fn add_group<'f>(&'f mut self, name: &str) -> error::Result<GroupMut<'f>> {
        let cname = std::ffi::CString::new(name).unwrap();
        let mut grpid = 0;
        unsafe {
            error::checked(nc_def_grp(self.ncid(), cname.as_ptr(), &mut grpid))?;
        }

        Ok(GroupMut(
            Group {
                ncid: grpid,
                _file: PhantomData,
            },
            PhantomData,
        ))
    }
    pub fn add_string_variable(&mut self, name: &str, dims: &[&str]) -> error::Result<VariableMut> {
        VariableMut::add_from_str((self.0).0.ncid, NC_STRING, name, dims)
    }
    pub fn variable_mut<'g>(&'g mut self, name: &str) -> Option<VariableMut<'g, 'g>> {
        let cname = std::ffi::CString::new(name).unwrap();
        let mut varid = 0;
        unsafe {
            error::checked(nc_inq_varid(self.ncid(), cname.as_ptr(), &mut varid)).unwrap();
        }
        let mut ndims = 0;
        let mut xtype = 0;
        let ncid = self.ncid();
        unsafe {
            error::checked(nc_inq_var(
                ncid,
                varid,
                std::ptr::null_mut(),
                &mut xtype,
                &mut ndims,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ))
            .unwrap();
        }
        let mut dimids = vec![0; ndims as _];
        unsafe {
            error::checked(nc_inq_vardimid(ncid, varid, dimids.as_mut_ptr())).unwrap();
        }

        let dimensions = dimids
            .into_iter()
            .map(|dimid| {
                let mut dimlen = 0;
                unsafe {
                    error::checked(nc_inq_dimlen(ncid, dimid, &mut dimlen))?;
                }
                Ok(Dimension {
                    len: core::num::NonZeroUsize::new(dimlen),
                    id: Identifier { ncid: ncid, dimid },
                    _group: PhantomData,
                })
            })
            .collect::<error::Result<Vec<_>>>()
            .unwrap();

        Some(VariableMut(
            Variable {
                ncid: self.ncid(),
                varid,
                dimensions,
                vartype: xtype,
                _group: PhantomData,
            },
            PhantomData,
        ))
    }
    pub fn variables_mut<'f>(
        &'f mut self,
    ) -> error::Result<impl Iterator<Item = VariableMut<'f, 'f>>> {
        self.variables()
            .map(|v| v.map(|var| VariableMut(var.unwrap(), PhantomData)))
    }
    pub fn add_attribute<'a, T>(&'a mut self, name: &str, val: T) -> error::Result<Attribute<'a>>
    where
        T: Into<AttrValue>,
    {
        Attribute::put((self.0).0.ncid, NC_GLOBAL, name, val.into())
    }
}

#[cfg(feature = "memory")]
/// The memory mapped file is kept in this structure to keep the
/// lifetime of the buffer longer than the file.
///
/// Access a [`ReadOnlyFile`] through the `Deref` trait,
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let buffer = &[0, 42, 1, 2];
/// let file = &netcdf::open_mem(None, buffer)?;
///
/// let variables = file.variables()?;
/// # Ok(()) }
/// ```
#[allow(clippy::module_name_repetitions)]
pub struct MemFile<'buffer>(ReadOnlyFile, std::marker::PhantomData<&'buffer [u8]>);

#[cfg(feature = "memory")]
impl<'a> std::ops::Deref for MemFile<'a> {
    type Target = ReadOnlyFile;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
