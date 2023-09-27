use foreign_types::ForeignTypeRef;

pub(crate) fn fake_drop<T>(_: *mut T) {
    unreachable!()
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
