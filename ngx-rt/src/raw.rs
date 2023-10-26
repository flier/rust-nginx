use std::ffi::{c_char, CStr, CString};
use std::ptr::NonNull;

use foreign_types::{ForeignType, ForeignTypeRef};

use crate::{
    core::conf::{NGX_CONF_ERROR, NGX_CONF_OK},
    Error,
};

#[inline(always)]
pub(crate) fn never_drop<T>(_: *mut T) {
    unreachable!()
}

pub(crate) unsafe fn box_copy<T>(p: *mut T) -> *mut T {
    Box::into_raw(Box::new(p.read()))
}

pub(crate) unsafe fn box_drop<T>(p: *mut T) {
    let _ = Box::from_raw(p);
}

pub trait NativeCallback {
    type CType;
}

/// Get a wrapped type for the raw type.
///
/// # Safety
///
/// This function is unsafe because the caller must ensure
/// that `ptr` is valid for `T` for the duration of the call.
/// The caller must not free `ptr` after the call.
pub unsafe trait FromRaw: ForeignType {
    /// Get a raw pointer to the type.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the caller must ensure
    /// that `ptr` is valid for `T` for the duration of the call.
    /// The caller must not free `ptr` after the call.
    unsafe fn from_raw(ptr: *mut Self::CType) -> Option<Self>;
}

unsafe impl<T: ForeignType> FromRaw for T {
    #[inline(always)]
    unsafe fn from_raw(ptr: *mut Self::CType) -> Option<Self> {
        NonNull::new(ptr).map(|p| unsafe { Self::from_ptr(p.as_ptr()) })
    }
}

/// Get a reference of foreign type from the raw ptr.
///
/// # Safety
///
/// This function is unsafe because the caller must ensure
/// that `ptr` is valid for `T` for the duration of the call.
/// The caller must not free `ptr` after the call.
pub unsafe trait FromRawRef<'a>: ForeignTypeRef {
    /// Get a raw reference to the type.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the caller must ensure
    /// that `ptr` is valid for `Self` for the duration of the call.
    /// The caller must ensure the reference is life long enough.
    unsafe fn from_raw(ptr: *mut Self::CType) -> Option<&'a Self>;
}

unsafe impl<'a, T: ForeignTypeRef> FromRawRef<'a> for T {
    #[inline(always)]
    unsafe fn from_raw(ptr: *mut Self::CType) -> Option<&'a Self> {
        NonNull::new(ptr).map(|p| unsafe { Self::from_ptr(p.as_ptr()) })
    }
}

/// Get a mutable reference of foreign type from the raw ptr.
///
/// # Safety
///
/// This function is unsafe because the caller must ensure
/// that `ptr` is valid for `T` for the duration of the call.
/// The caller must not free `ptr` after the call.
pub unsafe trait FromRawMut<'a>: ForeignTypeRef {
    /// Get a mutable raw reference to the type.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the caller must ensure
    /// that `ptr` is valid for `Self` for the duration of the call.
    /// The caller must ensure the reference is life long enough.
    unsafe fn from_raw_mut(ptr: *mut Self::CType) -> Option<&'a mut Self>;
}

unsafe impl<'a, T: ForeignTypeRef> FromRawMut<'a> for T {
    #[inline(always)]
    unsafe fn from_raw_mut(ptr: *mut Self::CType) -> Option<&'a mut Self> {
        NonNull::new(ptr).map(|p| unsafe { Self::from_ptr_mut(p.as_ptr()) })
    }
}

/// get a reference of raw type from the foreign type.
///
/// # Safety
///
/// This function is unsafe because the caller must ensure
/// that the foreign type is valid for the duration of the call.
pub unsafe trait AsRawRef: ForeignTypeRef {
    /// Get the raw reference to the type.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe fn as_raw(&self) -> &Self::CType;
}

unsafe impl<T: ForeignTypeRef> AsRawRef for T {
    #[inline(always)]
    unsafe fn as_raw(&self) -> &Self::CType {
        &*self.as_ptr()
    }
}

/// get a mutable reference of raw type from the foreign type.
///
/// # Safety
///
/// This function is unsafe because the caller must ensure
/// that the foreign type is valid for the duration of the call.
pub unsafe trait AsRawMut: ForeignTypeRef {
    /// Get the mutable raw reference to the type.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe fn as_raw_mut(&mut self) -> &mut Self::CType;
}

unsafe impl<T: ForeignTypeRef> AsRawMut for T {
    #[inline(always)]
    unsafe fn as_raw_mut(&mut self) -> &mut Self::CType {
        &mut *self.as_ptr()
    }
}

pub trait AsResult {
    type Output;
    type Error;

    fn ok(self) -> Result<Self::Output, Self::Error>
    where
        Self: Copy;

    fn ok_or(self, err: Self::Error) -> Result<Self::Output, Self::Error>
    where
        Self: Sized,
    {
        self.ok_or_else(|_| err)
    }

    fn ok_or_else<F>(self, err: F) -> Result<Self::Output, Self::Error>
    where
        Self: Sized,
        F: FnOnce(Self) -> Self::Error;
}

impl AsResult for ffi::ngx_int_t {
    type Output = Self;
    type Error = Self;

    fn ok(self) -> Result<Self::Output, Self::Error>
    where
        Self: Copy,
    {
        self.ok_or(self)
    }

    fn ok_or_else<F>(self, err: F) -> Result<Self::Output, Self::Error>
    where
        Self: Sized,
        F: FnOnce(Self) -> Self::Error,
    {
        if self == ffi::NGX_OK as isize {
            Ok(self)
        } else {
            Err(err(self))
        }
    }
}

impl AsResult for *mut c_char {
    type Output = ();
    type Error = Error;

    fn ok(self) -> Result<Self::Output, Self::Error>
    where
        Self: Copy,
    {
        self.ok_or_else(|err| {
            Error::ConfigError(if err == NGX_CONF_ERROR {
                CString::new("error").unwrap()
            } else {
                unsafe { CStr::from_ptr(self as *const _).to_owned() }
            })
        })
    }

    fn ok_or_else<F>(self, err: F) -> Result<(), Self::Error>
    where
        Self: Sized,
        F: FnOnce(Self) -> Self::Error,
    {
        if self == NGX_CONF_OK {
            Ok(())
        } else {
            Err(err(self))
        }
    }
}
