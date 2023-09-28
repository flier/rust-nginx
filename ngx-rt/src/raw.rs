use foreign_types::ForeignTypeRef;

pub(crate) fn never_drop<T>(_: *mut T) {
    unreachable!()
}

pub trait FromRaw: ForeignTypeRef {
    unsafe fn from_raw(ptr: *mut Self::CType) -> Option<&'static Self>;
}

impl<T: ForeignTypeRef> FromRaw for T {
    unsafe fn from_raw(ptr: *mut Self::CType) -> Option<&'static Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self::from_ptr(ptr))
        }
    }
}

pub trait AsRaw: ForeignTypeRef {
    /// Get the raw reference to the type.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe fn as_raw(&self) -> &Self::CType;

    /// Get the mutable raw reference to the type.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe fn as_raw_mut(&mut self) -> &mut Self::CType;
}

impl<T: ForeignTypeRef> AsRaw for T {
    unsafe fn as_raw(&self) -> &Self::CType {
        &*self.as_ptr()
    }

    unsafe fn as_raw_mut(&mut self) -> &mut Self::CType {
        &mut *self.as_ptr()
    }
}
