use std::{ffi::c_char, ptr::NonNull};

use foreign_types::{ForeignType, ForeignTypeRef};

#[inline(always)]
pub(crate) fn never_drop<T>(_: *mut T) {
    unreachable!()
}

pub trait NativeCallback {
    type CType;
}

pub trait FromRaw: ForeignType {
    /// Get a raw pointer to the type.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the caller must ensure
    /// that `ptr` is valid for `T` for the duration of the call.
    /// The caller must not free `ptr` after the call.
    unsafe fn from_raw(ptr: *mut Self::CType) -> Option<Self>;
}

impl<T: ForeignType> FromRaw for T {
    #[inline(always)]
    unsafe fn from_raw(ptr: *mut Self::CType) -> Option<Self> {
        NonNull::new(ptr).map(|p| unsafe { Self::from_ptr(p.as_ptr()) })
    }
}

pub trait FromRawRef<'a>: ForeignTypeRef {
    /// Get a raw reference to the type.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the caller must ensure
    /// that `ptr` is valid for `Self` for the duration of the call.
    /// The caller must ensure the reference is life long enough.
    unsafe fn from_raw(ptr: *mut Self::CType) -> Option<&'a Self>;
}

impl<'a, T: ForeignTypeRef> FromRawRef<'a> for T {
    #[inline(always)]
    unsafe fn from_raw(ptr: *mut Self::CType) -> Option<&'a Self> {
        NonNull::new(ptr).map(|p| unsafe { Self::from_ptr(p.as_ptr()) })
    }
}

pub trait FromRawMut<'a>: ForeignTypeRef {
    /// Get a mutable raw reference to the type.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the caller must ensure
    /// that `ptr` is valid for `Self` for the duration of the call.
    /// The caller must ensure the reference is life long enough.
    unsafe fn from_raw_mut(ptr: *mut Self::CType) -> Option<&'a mut Self>;
}

impl<'a, T: ForeignTypeRef> FromRawMut<'a> for T {
    #[inline(always)]
    unsafe fn from_raw_mut(ptr: *mut Self::CType) -> Option<&'a mut Self> {
        NonNull::new(ptr).map(|p| unsafe { Self::from_ptr_mut(p.as_ptr()) })
    }
}

pub trait AsRawRef: ForeignTypeRef {
    /// Get the raw reference to the type.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe fn as_raw(&self) -> &Self::CType;
}

impl<T: ForeignTypeRef> AsRawRef for T {
    #[inline(always)]
    unsafe fn as_raw(&self) -> &Self::CType {
        &*self.as_ptr()
    }
}

pub trait AsRawMut: ForeignTypeRef {
    /// Get the mutable raw reference to the type.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe fn as_raw_mut(&mut self) -> &mut Self::CType;
}

impl<T: ForeignTypeRef> AsRawMut for T {
    #[inline(always)]
    unsafe fn as_raw_mut(&mut self) -> &mut Self::CType {
        &mut *self.as_ptr()
    }
}

pub trait AsResult {
    fn ok(self) -> Result<Self, Self>
    where
        Self: Copy,
    {
        self.ok_or(self)
    }

    fn ok_or<E>(self, err: E) -> Result<Self, E>
    where
        Self: Sized,
    {
        self.ok_or_else(|_| err)
    }

    fn ok_or_else<E, F>(self, err: F) -> Result<Self, E>
    where
        Self: Sized,
        F: FnOnce(Self) -> E;
}

impl AsResult for ffi::ngx_int_t {
    fn ok_or_else<E, F>(self, err: F) -> Result<Self, E>
    where
        Self: Sized,
        F: FnOnce(Self) -> E,
    {
        if self == ffi::NGX_OK as isize {
            Ok(self)
        } else {
            Err(err(self))
        }
    }
}

impl AsResult for *mut c_char {
    fn ok_or_else<E, F>(self, err: F) -> Result<Self, E>
    where
        Self: Sized,
        F: FnOnce(Self) -> E,
    {
        if self != usize::MAX as Self {
            Ok(self)
        } else {
            Err(err(self))
        }
    }
}
