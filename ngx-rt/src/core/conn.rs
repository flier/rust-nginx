use std::marker::PhantomData;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::os::fd::{AsRawFd, RawFd};
use std::ptr::{null_mut, NonNull};
use std::{mem, slice};

use foreign_types::{foreign_type, ForeignTypeRef};
use num_enum::FromPrimitive;

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
        unsafe {
            let r = self.as_raw();

            NonNull::new(r.sockaddr).and_then(|p| sockaddr(p, r.socklen as usize))
        }
    }
}

impl AsRef<LogRef> for ListeningRef {
    fn as_ref(&self) -> &LogRef {
        self.log()
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

        log_error() into LogError;
        tcp_nodelay() into TcpNoDelay;
        tcp_nopush() into TcpNoPush;
    }

    str! {
        &addr_text?;
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

    pub fn ty(&self) -> SocketType {
        SocketType(unsafe { self.as_raw().type_ })
    }

    pub fn remote(&self) -> Option<SocketAddr> {
        unsafe {
            let r = self.as_raw();

            NonNull::new(r.sockaddr).and_then(|p| sockaddr(p, r.socklen as usize))
        }
    }

    pub fn local(&self) -> Option<SocketAddr> {
        unsafe {
            if ffi::ngx_connection_local_sockaddr(self.as_ptr(), null_mut(), 0)
                == ffi::NGX_OK as isize
            {
                let r = self.as_raw();
                NonNull::new(r.local_sockaddr).and_then(|p| sockaddr(p, r.local_socklen as usize))
            } else {
                None
            }
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

impl AsRef<LogRef> for ConnRef {
    fn as_ref(&self) -> &LogRef {
        self.log()
    }
}

impl AsRawFd for ConnRef {
    fn as_raw_fd(&self) -> RawFd {
        unsafe { self.as_raw().fd }
    }
}

unsafe fn sockaddr(sa: NonNull<ffi::sockaddr>, len: usize) -> Option<SocketAddr> {
    match sa.as_ref().sa_family as i32 {
        libc::AF_INET if len >= mem::size_of::<libc::sockaddr_in>() => sa
            .as_ptr()
            .cast_const()
            .cast::<libc::sockaddr_in>()
            .as_ref()
            .map(|sa| {
                SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::from(sa.sin_addr.s_addr.to_ne_bytes()),
                    u16::from_be(sa.sin_port),
                ))
            }),
        libc::AF_INET6 if len >= mem::size_of::<libc::sockaddr_in6>() => sa
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

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SocketType(i32);

impl SocketType {
    /// Type corresponding to `SOCK_STREAM`.
    ///
    /// Used for protocols such as TCP.
    pub const STREAM: SocketType = SocketType(libc::SOCK_STREAM);

    /// Type corresponding to `SOCK_DGRAM`.
    ///
    /// Used for protocols such as UDP.
    pub const DGRAM: SocketType = SocketType(libc::SOCK_DGRAM);

    /// Type corresponding to SOCK_RAW.
    pub const RAW: SocketType = SocketType(libc::SOCK_RAW);

    /// Type corresponding to `SOCK_SEQPACKET`.
    pub const SEQPACKET: SocketType = SocketType(libc::SOCK_SEQPACKET);
}

impl SocketType {
    pub fn is_stream(&self) -> bool {
        *self == Self::STREAM
    }

    pub fn is_dgram(&self) -> bool {
        *self == Self::DGRAM
    }

    pub fn is_raw(&self) -> bool {
        *self == Self::RAW
    }

    pub fn is_seq_packet(&self) -> bool {
        *self == Self::SEQPACKET
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, FromPrimitive)]
pub enum LogError {
    #[default]
    Alert = 0,
    Error,
    Info,
    IgnoreConnReset,
    IgnoreInvalid,
    IgnoreMsgSize,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, FromPrimitive)]
pub enum TcpNoDelay {
    #[default]
    Unset,
    Set,
    Disabled,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, FromPrimitive)]
pub enum TcpNoPush {
    #[default]
    Unset,
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
