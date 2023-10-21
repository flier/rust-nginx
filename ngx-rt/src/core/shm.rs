use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
    slice,
};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{core::LogRef, ffi, never_drop, AsRawMut, AsRawRef};

foreign_type! {
    pub unsafe type Shm: Send {
        type CType = ffi::ngx_shm_t;

        fn drop = ffi::ngx_shm_free;
    }
}

impl ShmRef {
    const EXISTS_BIT: usize = 0x0001;

    str! {
        &name;
    }

    property! {
        size: usize;
        log: &LogRef;
    }

    pub fn exists(&self) -> bool {
        unsafe { self.as_raw().exists & Self::EXISTS_BIT != 0 }
    }

    pub fn addr(&self) -> Option<NonNull<u8>> {
        NonNull::new(unsafe { self.as_raw().addr.cast() })
    }

    pub fn as_slice(&self) -> Option<&[u8]> {
        unsafe {
            let r = self.as_raw();

            NonNull::new(r.addr).map(|p| slice::from_raw_parts(p.cast().as_ptr(), r.size))
        }
    }

    pub fn as_slice_mut(&mut self) -> Option<&mut [u8]> {
        unsafe {
            let r = self.as_raw();

            NonNull::new(r.addr).map(|p| slice::from_raw_parts_mut(p.cast().as_ptr(), r.size))
        }
    }
}

impl AsRef<[u8]> for ShmRef {
    fn as_ref(&self) -> &[u8] {
        self.as_slice().unwrap()
    }
}

impl AsMut<[u8]> for ShmRef {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_slice_mut().unwrap()
    }
}

impl Deref for ShmRef {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice().unwrap()
    }
}

impl DerefMut for ShmRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut().unwrap()
    }
}

foreign_type! {
    pub unsafe type Zone: Send {
        type CType = ffi::ngx_shm_zone_t;

        fn drop = never_drop::<ffi::ngx_shm_zone_t>;
    }
}

impl ZoneRef {}

impl Deref for ZoneRef {
    type Target = ShmRef;

    fn deref(&self) -> &Self::Target {
        unsafe { ShmRef::from_ptr(&self.as_raw().shm as *const _ as *mut _) }
    }
}

impl DerefMut for ZoneRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { ShmRef::from_ptr_mut(&mut self.as_raw_mut().shm) }
    }
}
