use std::slice;
use std::{marker::PhantomData, ptr::NonNull};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{ffi, flag, never_drop, property, AsRawRef};

use super::{BufRef, LogRef, PoolRef};

foreign_type! {
    pub unsafe type Conn: Send {
        type CType = ffi::ngx_connection_t;

        fn drop = never_drop::<ffi::ngx_connection_t>;
    }
}

impl ConnRef {
    property! {
        log: &LogRef;
        pool: &PoolRef;
        buffer: &BufRef;
        requests: usize;
    }

    flag! {
        timedout();
        error();
        destroyed();
        pipeline();

        idle();
        reusable();
        close();
        shared();

        sendfile();
        sndlowat();

        need_last_buf();
        need_flush_buf();
    }

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

    pub fn tcp_nodelay(&self) -> Option<TcpNoDelay> {
        match unsafe { self.as_raw().tcp_nodelay() } {
            1 => Some(TcpNoDelay::Set),
            2 => Some(TcpNoDelay::Disabled),
            _ => None,
        }
    }

    pub fn tcp_nopush(&self) -> Option<TcpNoPush> {
        match unsafe { self.as_raw().tcp_nopush() } {
            1 => Some(TcpNoPush::Set),
            2 => Some(TcpNoPush::Disabled),
            _ => None,
        }
    }
}

pub enum LogError {
    Alert = 0,
    Error,
    Info,
    IgnoreConnReset,
    IgnoreInvalid,
    IgnoreMsgSize,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TcpNoDelay {
    Set = 1,
    Disabled,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TcpNoPush {
    Set = 1,
    Disabled,
}

pub struct ConnSlice<'a>(pub(crate) &'a [ffi::ngx_connection_t]);

impl<'a> IntoIterator for &'a ConnSlice<'a> {
    type Item = &'a ConnRef;
    type IntoIter = ConnsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ConnsIter(self.0.iter())
    }
}

pub struct ConnsIter<'a>(slice::Iter<'a, ffi::ngx_connection_t>);

impl<'a> Iterator for ConnsIter<'a> {
    type Item = &'a ConnRef;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|c| unsafe { ConnRef::from_ptr(c as *const _ as *mut _) })
    }
}

pub struct ConnList<'a> {
    next: Option<NonNull<ffi::ngx_connection_t>>,
    n: usize,
    phantom: PhantomData<&'a u8>,
}

impl<'a> ConnList<'a> {
    pub fn new(p: Option<NonNull<ffi::ngx_connection_t>>, n: usize) -> Self {
        ConnList {
            next: p,
            n,
            phantom: PhantomData,
        }
    }
}

impl<'a> Iterator for ConnList<'a> {
    type Item = &'a ConnRef;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.next.take();

        if let Some(p) = c {
            self.next = NonNull::new(unsafe { p.as_ref().data.cast() });
            self.n -= 1;

            Some(unsafe { ConnRef::from_ptr(p.as_ptr()) })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.n, Some(self.n))
    }
}
