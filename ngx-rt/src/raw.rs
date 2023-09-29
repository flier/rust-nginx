use std::ptr::NonNull;

use foreign_types::{ForeignType, ForeignTypeRef};

#[inline(always)]
pub(crate) fn never_drop<T>(_: *mut T) {
    unreachable!()
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
