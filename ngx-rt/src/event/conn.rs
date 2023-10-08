use std::{
    fmt,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{
    core::{ConnRef, LogError, LogRef},
    ffi, flag, native_callback, never_drop, property, AsRawMut, AsRawRef, Error,
};

foreign_type! {
    pub unsafe type PeerConn: Send {
        type CType = ffi::ngx_peer_connection_t;

        fn drop = never_drop::<ffi::ngx_peer_connection_t>;
    }
}

impl fmt::Pointer for PeerConnRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let p = self.as_ptr();

        fmt::Pointer::fmt(&p, f)
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

    property!(tries: usize);

    pub fn get(&self) -> Option<GetPeerFn> {
        unsafe { self.as_raw().get.map(GetPeerFn) }
    }

    pub fn free(&self) -> Option<FreePeerFn> {
        unsafe { self.as_raw().free.map(FreePeerFn) }
    }

    pub fn notify(&self) -> Option<NotifyPeerFn> {
        unsafe { self.as_raw().notify.map(NotifyPeerFn) }
    }

    pub fn data<T>(&self) -> Option<&T> {
        unsafe { NonNull::new(self.as_raw().data.cast()).map(|p| p.as_ref()) }
    }

    property!(type_: i32);
    property!(rcvbuf: i32);
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

#[native_callback]
pub type GetPeerFn<T> = fn(pc: &PeerConnRef, data: Option<&T>) -> Result<(), Error>;

#[native_callback]
pub type FreePeerFn<T> = fn(pc: &PeerConnRef, data: Option<&T>, state: usize);

#[native_callback]
pub type NotifyPeerFn<T> = fn(pc: &PeerConnRef, data: Option<&T>, ty: usize);
