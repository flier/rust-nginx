use std::{
    ffi::c_void,
    ops::{Deref, DerefMut},
    ptr,
};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{
    core::{ConnRef, LogError, LogRef},
    ffi, flag, never_drop, property, AsRawMut, AsRawRef, AsResult,
};

foreign_type! {
    pub unsafe type PeerConn: Send {
        type CType = ffi::ngx_peer_connection_t;

        fn drop = never_drop::<ffi::ngx_peer_connection_t>;
    }
}

impl Deref for PeerConnRef {
    type Target = <Self as ForeignTypeRef>::CType;

    fn deref(&self) -> &Self::Target {
        unsafe { self.as_raw() }
    }
}

impl DerefMut for PeerConnRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.as_raw_mut() }
    }
}

impl PeerConnRef {
    property!(connection: &ConnRef);

    pub fn get(&self) -> Option<GetPeerFn> {
        unsafe { self.as_raw().get.map(GetPeerFn) }
    }

    pub fn free(&self) -> Option<FreePeerFn> {
        unsafe { self.as_raw().free.map(FreePeerFn) }
    }

    property!(tries: usize);
    property!(log: &LogRef);

    flag!(cached());
    flag!(transparent());
    flag!(so_keepalive());
    flag!(down());

    pub fn log_error(&self) -> LogError {
        match unsafe { self.as_raw().log_error() } {
            0 => LogError::Alert,
            1 => LogError::Error,
            2 => LogError::Info,
            3 => LogError::IgnoreConnReset,
            4 => LogError::IgnoreInvalid,
            5 => LogError::IgnoreMsgSize,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct GetPeerFn(
    pub  unsafe extern "C" fn(
        pc: *mut ffi::ngx_peer_connection_t,
        data: *mut c_void,
    ) -> ffi::ngx_int_t,
);

impl GetPeerFn {
    pub fn call<T>(&self, pc: &PeerConnRef, data: Option<&T>) -> Result<isize, isize> {
        unsafe {
            self.0(
                pc.as_ptr(),
                data.map_or(ptr::null_mut(), |p| p as *const _ as *mut T as *mut _),
            )
        }
        .ok()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct FreePeerFn(
    pub  unsafe extern "C" fn(
        pc: *mut ffi::ngx_peer_connection_t,
        data: *mut c_void,
        state: ffi::ngx_uint_t,
    ),
);

impl FreePeerFn {
    pub fn call<T>(&self, pc: &PeerConnRef, data: Option<&T>, state: usize) {
        unsafe {
            self.0(
                pc.as_ptr(),
                data.map_or(ptr::null_mut(), |p| p as *const _ as *mut T as *mut _),
                state,
            )
        }
    }
}
