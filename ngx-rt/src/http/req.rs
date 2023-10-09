use std::{ffi::CStr, ops::Deref, ptr::NonNull, slice};

use bitflags::bitflags;
use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{
    core::{BufRef, ConnRef, PoolRef, Str},
    ffi, flag, never_drop, property, str, AsRawRef,
};

use super::{upstream::UpstreamRef, UnsafeLocConf, UnsafeMainConf, UnsafeSrvConf};

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Method : u32 {
        const UNKNOWN = ffi::NGX_HTTP_UNKNOWN;
        const GET = ffi::NGX_HTTP_GET;
        const HEAD = ffi::NGX_HTTP_HEAD;
        const POST = ffi::NGX_HTTP_POST;
        const PUT = ffi::NGX_HTTP_PUT;
        const DELETE = ffi::NGX_HTTP_DELETE;
        const MKCOL = ffi::NGX_HTTP_MKCOL;
        const COPY = ffi::NGX_HTTP_COPY;
        const MOVE = ffi::NGX_HTTP_MOVE;
        const OPTIONS = ffi::NGX_HTTP_OPTIONS;
        const PROPFIND = ffi::NGX_HTTP_PROPFIND;
        const PROPPATCH = ffi::NGX_HTTP_PROPPATCH;
        const LOCK = ffi::NGX_HTTP_LOCK;
        const UNLOCK = ffi::NGX_HTTP_UNLOCK;
        const PATCH = ffi::NGX_HTTP_PATCH;
        const TRACE = ffi::NGX_HTTP_TRACE;
        const CONNECT = ffi::NGX_HTTP_CONNECT;
    }
}

impl Method {
    fn as_method(&self) -> Option<::http::Method> {
        Some(match *self {
            Method::GET => ::http::Method::GET,
            Method::HEAD => ::http::Method::HEAD,
            Method::POST => ::http::Method::POST,
            Method::PUT => ::http::Method::PUT,
            Method::DELETE => ::http::Method::DELETE,
            Method::MKCOL => ::http::Method::from_bytes(b"MKCOL").unwrap(),
            Method::COPY => ::http::Method::from_bytes(b"COPY").unwrap(),
            Method::MOVE => ::http::Method::from_bytes(b"MOVE").unwrap(),
            Method::OPTIONS => ::http::Method::OPTIONS,
            Method::PROPFIND => ::http::Method::from_bytes(b"PROPFIND").unwrap(),
            Method::PROPPATCH => ::http::Method::from_bytes(b"PROPPATCH").unwrap(),
            Method::LOCK => ::http::Method::from_bytes(b"LOCK").unwrap(),
            Method::UNLOCK => ::http::Method::from_bytes(b"UNLOCK").unwrap(),
            Method::PATCH => ::http::Method::PATCH,
            Method::TRACE => ::http::Method::TRACE,
            Method::CONNECT => ::http::Method::CONNECT,
            _ => return None,
        })
    }
}

macro_rules! header {
    () => {};

    ($(#[$attr:meta])* $name:ident) => {
        $crate::property!($(#[$attr])* $name as Header);
    };

    ($(#[$attr:meta])* $name:ident ; $($rest:tt)* ) => {
        header!($(#[$attr])* $name);
        header!( $($rest)* );
    };
}

foreign_type! {
    pub unsafe type Request: Send {
        type CType = ffi::ngx_http_request_t;

        fn drop = never_drop::<ffi::ngx_http_request_t>;
    }
}

impl Deref for RequestRef {
    type Target = HeadersInRef;

    fn deref(&self) -> &Self::Target {
        self.headers_in()
    }
}

impl UnsafeMainConf for RequestRef {
    unsafe fn main_conf<T>(&self, idx: usize) -> Option<&mut T> {
        NonNull::new(self.as_raw().main_conf.add(idx).read()).map(|p| p.cast::<T>().as_mut())
    }
}

impl UnsafeSrvConf for RequestRef {
    unsafe fn srv_conf<T>(&self, idx: usize) -> Option<&mut T> {
        NonNull::new(self.as_raw().srv_conf.add(idx).read()).map(|p| p.cast::<T>().as_mut())
    }
}

impl UnsafeLocConf for RequestRef {
    unsafe fn loc_conf<T>(&self, idx: usize) -> Option<&mut T> {
        NonNull::new(self.as_raw().loc_conf.add(idx).read()).map(|p| p.cast::<T>().as_mut())
    }
}

impl RequestRef {
    property! {
        connection: &ConnRef;
        upstream as &mut UpstreamRef;
        pool: &PoolRef;
        header_in: &BufRef;
        &headers_in: &HeadersInRef;
        &headers_out: &HeadersOutRef;
        request_body as &BodyRef;
        main as &Self;
        parent as &Self;
    }

    str! {
        request_line;
        uri;
        args;
        exten;
        unparsed_uri;
        method_name;
        http_protocol;
        schema;
    }

    pub fn method(&self) -> Method {
        Method::from_bits_truncate(unsafe { self.as_raw().method as u32 })
    }

    pub fn as_method(&self) -> Option<http::Method> {
        self.method().as_method().or_else(|| {
            http::Method::from_bytes(unsafe {
                let r = self.as_raw();

                slice::from_raw_parts(
                    r.request_start,
                    r.method_end.offset_from(r.request_start) as usize,
                )
            })
            .ok()
        })
    }

    pub fn version(&self) -> (u32, u32) {
        unsafe {
            let v = self.as_raw().http_version;

            ((v >> 16) as u32, (v & 0xFFFF) as u32)
        }
    }
}

foreign_type! {
    pub unsafe type HeadersIn: Send {
        type CType = ffi::ngx_http_headers_in_t;

        fn drop = never_drop::<ffi::ngx_http_headers_in_t>;
    }
}

impl HeadersInRef {
    header! {
        authorization;
        connection;
        content_length;
        content_range;
        content_type;
        cookie;
        expect;
        host;
        if_match;
        if_modified_since;
        if_none_match;
        if_range;
        if_unmodified_since;
        keep_alive;
        range;
        referer;
        te;
        transfer_encoding;
        upgrade;
        user_agent;
    }

    #[cfg(any(feature = "http_gzip", feature = "http_headers"))]
    header! {
        accept_encoding;
        via;
    }

    #[cfg(feature = "http_x_forwarded_for")]
    header!(x_forwarded_for);

    #[cfg(feature = "http_realip")]
    header!(x_real_ip);

    #[cfg(feature = "http_headers")]
    header! {
        accept;
        accept_language;
    }

    #[cfg(feature = "http_dav")]
    header! {
        depth;
        destination;
        overwrite;
        date;
    }

    str! {
        user;
        passwd;
        server;
    }

    property! {
        connection_type() as ConnectionType;
        content_length_n: i64;
        headers: Headers;
        keep_alive_n: i64;
    }

    flag! {
        chunked();
        multi();
        multi_linked();
        msie();
        msie6();
        opera();
        gecko();
        chrome();
        safari();
        konqueror();
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionType {
    Close = ffi::NGX_HTTP_CONNECTION_CLOSE,
    KeepAlive = ffi::NGX_HTTP_CONNECTION_KEEP_ALIVE,
}

impl ConnectionType {
    pub fn from_raw(n: u32) -> Option<Self> {
        match n {
            ffi::NGX_HTTP_CONNECTION_CLOSE => Some(ConnectionType::Close),
            ffi::NGX_HTTP_CONNECTION_KEEP_ALIVE => Some(ConnectionType::KeepAlive),
            _ => None,
        }
    }
}

foreign_type! {
    pub unsafe type HeadersOut: Send {
        type CType = ffi::ngx_http_headers_out_t;

        fn drop = never_drop::<ffi::ngx_http_headers_out_t>;
    }
}

impl HeadersOutRef {
    property! {
        headers: Headers;
        trailers: Headers;

        status: usize;
        status_line: Str;

        content_type_len: usize;
        content_type: Str;
        charset: Str;

        content_type_hash: usize;
        content_length_n: i64;
        content_offset: i64;
        date_time: i64;
        last_modified_time: i64;
    }

    header! {
        accept_ranges;
        cache_control;
        content_encoding;
        content_length;
        content_range;
        date;
        etag;
        expires;
        last_modified;
        link;
        location;
        refresh;
        server;
        www_authenticate;
    }

    pub fn override_charset(&self) -> Option<Str> {
        unsafe { Str::from_ptr(self.as_raw().override_charset) }
    }

    pub fn content_type_lowcase(&self) -> Option<&CStr> {
        unsafe {
            NonNull::new(self.as_raw().content_type_lowcase)
                .map(|p| CStr::from_ptr(p.as_ptr() as *const _))
        }
    }
}

foreign_type! {
    pub unsafe type Body: Send {
        type CType = ffi::ngx_http_request_body_t;

        fn drop = never_drop::<ffi::ngx_http_request_body_t>;
    }
}

impl BodyRef {
    property! {
        rest: i64;
        received: i64;
    }

    flag! {
        filter_need_buffering();
        last_sent();
        last_saved();
    }
}
