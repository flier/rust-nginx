use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{core::ConfRef, ffi, http::RequestRef, never_drop, AsRawMut, AsRawRef, AsResult};

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

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct InitFn(
    pub  unsafe extern "C" fn(
        cf: *mut ffi::ngx_conf_t,
        us: *mut ffi::ngx_http_upstream_srv_conf_t,
    ) -> ffi::ngx_int_t,
);

impl InitFn {
    pub fn call(&self, cf: &ConfRef, us: &SrvConfRef) -> Result<isize, isize> {
        unsafe { self.0(cf.as_ptr(), us.as_ptr()) }.ok()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct InitPeerFn(
    pub  unsafe extern "C" fn(
        r: *mut ffi::ngx_http_request_t,
        us: *mut ffi::ngx_http_upstream_srv_conf_t,
    ) -> ffi::ngx_int_t,
);

impl InitPeerFn {
    pub fn call(&self, r: &RequestRef, us: &SrvConfRef) -> Result<isize, isize> {
        unsafe { self.0(r.as_ptr(), us.as_ptr()) }.ok()
    }
}
