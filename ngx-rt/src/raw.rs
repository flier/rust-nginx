use foreign_types::ForeignTypeRef;

pub(crate) fn fake_drop<T>(_: *mut T) {
    unreachable!()
}

pub trait AsRaw: ForeignTypeRef {
    unsafe fn as_raw(&self) -> &Self::CType;

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
