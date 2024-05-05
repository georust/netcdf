use std::ffi::{c_char, CStr, CString};
use std::ops::Deref;
use std::ptr::NonNull;

pub fn set(key: &str, value: &str) -> crate::error::Result<()> {
    let key = CString::new(key)?;
    let value = CString::new(value)?;
    crate::error::checked(crate::with_lock(|| unsafe {
        netcdf_sys::nc_rc_set(key.as_ptr(), value.as_ptr())
    }))
}

#[derive(Debug)]
pub struct OwnedString {
    inner: NonNull<c_char>,
}

impl Deref for OwnedString {
    type Target = CStr;
    fn deref(&self) -> &Self::Target {
        unsafe { CStr::from_ptr(self.inner.as_ptr()) }
    }
}

impl Drop for OwnedString {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.inner.as_ptr().cast());
        }
    }
}

pub fn get(key: &str) -> Option<OwnedString> {
    let key = if let Ok(key) = CString::new(key) {
        key
    } else {
        return None;
    };
    let _lock = netcdf_sys::libnetcdf_lock.lock().unwrap();
    let value = unsafe { netcdf_sys::nc_rc_get(key.as_ptr()) };
    NonNull::new(value).map(|inner| OwnedString { inner })
}
