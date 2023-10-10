use std::marker::PhantomData;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::os::fd::{AsRawFd, RawFd};
use std::ptr::NonNull;
use std::slice;

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{ffi, flag, never_drop, property, AsRawRef, AsResult, Error};

use super::{BufRef, LogRef, PoolRef};

foreign_type! {
    pub unsafe type Listening: Send {
        type CType = ffi::ngx_listening_t;

        fn drop = never_drop::<ffi::ngx_listening_t>;
    }
}

impl ListeningRef {
    property! {
        backlog: i32;
        rcvbuf: i32;
        sndbuf: i32;
        keepidle: i32;
        keepintvl: i32;
        keepcnt: i32;

        &log: &LogRef;
        previous as &ListeningRef;
        connection as &ConnRef;
    }

    flag! {
        open;
        remain;
        ignore;

        /// already bound
        bound;
        /// inherited from previous process
        inherited;
        nonblocking_accept;
        listen;
        nonblocking;
        /// shared between threads or processes
        shared;
        addr_ntop;
        wildcard;

        ipv6only;
        reuseport;
        add_reuseport;

        deferred_accept;
        delete_deferred;
        add_deferred;
    }

    pub fn addr(&self) -> Option<SocketAddr> {
        unsafe { NonNull::new(self.as_raw().sockaddr).and_then(|p| sockaddr(p)) }
    }
}

impl AsRawFd for ListeningRef {
    fn as_raw_fd(&self) -> RawFd {
        unsafe { self.as_raw().fd }
    }
}

foreign_type! {
    pub unsafe type Conn: Send {
        type CType = ffi::ngx_connection_t;

        fn drop = ffi::ngx_free_connection;
    }
}

impl ConnRef {
    property! {
        listening: &ListeningRef;
        sent: i64;
        log: &LogRef;
        pool: &PoolRef;
        buffer: &BufRef;
        requests: usize;
    }

    flag! {
        timedout;
        error;
        destroyed;
        pipeline;

        idle;
        reusable;
        shared;

        sendfile;
        sndlowat;

        need_last_buf;
        need_flush_buf;
    }

    pub fn closed(&self) -> bool {
        unsafe { self.as_raw().close() != 0 }
    }

    pub fn remote(&self) -> Option<SocketAddr> {
        unsafe { NonNull::new(self.as_raw().sockaddr).and_then(|p| sockaddr(p)) }
    }

    pub fn local(&self) -> Option<SocketAddr> {
        unsafe { NonNull::new(self.as_raw().local_sockaddr).and_then(|p| sockaddr(p)) }
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

    pub fn close(&self) {
        unsafe { ffi::ngx_close_connection(self.as_ptr()) }
    }

    pub fn set_tcp_nodelay(&self) -> Result<(), Error> {
        unsafe {
            ffi::ngx_tcp_nodelay(self.as_ptr())
                .ok()
                .map(|_| ())
                .map_err(|_| Error::errno())
        }
    }

    pub fn set_reusable(&self, reusable: bool) {
        unsafe { ffi::ngx_reusable_connection(self.as_ptr(), if reusable { 1 } else { 0 }) }
    }
}

impl AsRawFd for ConnRef {
    fn as_raw_fd(&self) -> RawFd {
        unsafe { self.as_raw().fd }
    }
}

unsafe fn sockaddr(sa: NonNull<ffi::sockaddr>) -> Option<SocketAddr> {
    match sa.as_ref().sa_family as i32 {
        libc::AF_INET => sa
            .as_ptr()
            .cast_const()
            .cast::<libc::sockaddr_in>()
            .as_ref()
            .map(|sa| {
                SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::from(sa.sin_addr.s_addr.to_be_bytes()),
                    u16::from_be(sa.sin_port),
                ))
            }),
        libc::AF_INET6 => sa
            .as_ptr()
            .cast_const()
            .cast::<libc::sockaddr_in6>()
            .as_ref()
            .map(|sa| {
                SocketAddr::V6(SocketAddrV6::new(
                    Ipv6Addr::from(sa.sin6_addr.s6_addr),
                    u16::from_be(sa.sin6_port),
                    sa.sin6_flowinfo,
                    sa.sin6_scope_id,
                ))
            }),
        _ => None,
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
