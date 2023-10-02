use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{
    core::ConfRef, ffi, http::RequestRef, native_callback, never_drop, AsRawMut, AsRawRef, Error,
};

use super::SrvConfRef;

foreign_type! {
    pub unsafe type Peer: Send {
        type CType = ffi::ngx_http_upstream_peer_t;

        fn drop = never_drop::<ffi::ngx_http_upstream_peer_t>;
    }
}

impl Deref for PeerRef {
    type Target = <Self as ForeignTypeRef>::CType;

    fn deref(&self) -> &Self::Target {
        unsafe { self.as_raw() }
    }
}

impl DerefMut for PeerRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.as_raw_mut() }
    }
}

impl PeerRef {
    pub fn init_upstream(&self) -> Option<InitFn> {
        unsafe { self.as_raw().init_upstream.map(InitFn) }
    }

    pub fn init(&self) -> Option<InitPeerFn> {
        unsafe { self.as_raw().init.map(InitPeerFn) }
    }

    pub fn data<T>(&self) -> Option<NonNull<T>> {
        NonNull::new(unsafe { self.as_raw().data.cast() })
    }
}

#[native_callback]
pub type InitFn = fn(cf: &ConfRef, us: &SrvConfRef) -> Result<(), Error>;

#[native_callback]
pub type InitPeerFn = fn(r: &RequestRef, us: &SrvConfRef) -> Result<(), Error>;
