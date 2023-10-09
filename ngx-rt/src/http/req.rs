use std::{ffi::CStr, ops::Deref, ptr::NonNull, slice};

use bitflags::bitflags;
use cfg_if::cfg_if;
use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{
    core::{BufRef, ConnRef, ModuleRef, PoolRef, Str},
    ffi, flag, never_drop, property, str, AsRawRef,
};

use super::upstream::UpstreamRef;

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
    ($name:ident) => {
        property!($name as Header);
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

impl RequestRef {
    property!(connection: &ConnRef);

    /// Get the main configuration for the module.
    pub fn main_conf_for<T>(&self, m: &ModuleRef) -> Option<&mut T> {
        unsafe { self.main_conf(m.context_index()) }
    }

    /// Get the server configuration for the module.
    pub fn srv_conf_for<T>(&self, m: &ModuleRef) -> Option<&mut T> {
        unsafe { self.srv_conf(m.context_index()) }
    }

    /// Get the location configuration for the module.
    pub fn loc_conf_for<T>(&self, m: &ModuleRef) -> Option<&mut T> {
        unsafe { self.loc_conf(m.context_index()) }
    }

    /// Get the main configuration from context.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `main_conf` array.
    pub unsafe fn main_conf<T>(&self, idx: usize) -> Option<&mut T> {
        self.as_raw().main_conf.add(idx).read().cast::<T>().as_mut()
    }

    /// Get the server configuration from context.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `srv_conf` array.
    pub unsafe fn srv_conf<T>(&self, idx: usize) -> Option<&mut T> {
        self.as_raw().srv_conf.add(idx).read().cast::<T>().as_mut()
    }

    /// Get the location configuration from context.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `loc_conf` array.
    pub unsafe fn loc_conf<T>(&self, idx: usize) -> Option<&mut T> {
        self.as_raw().loc_conf.add(idx).read().cast::<T>().as_mut()
    }

    property!(upstream as &mut UpstreamRef);
    property!(pool: &PoolRef);
    property!(header_in: &BufRef);
    property!(&headers_in: &HeadersInRef);
    property!(&headers_out: &HeadersOutRef);
    property!(request_body as &BodyRef);

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

    str!(request_line);
    str!(uri);
    str!(args);
    str!(exten);
    str!(unparsed_uri);
    str!(method_name);
    str!(http_protocol);
    str!(schema);
    property!(main as &Self);
    property!(parent as &Self);
}

foreign_type! {
    pub unsafe type HeadersIn: Send {
        type CType = ffi::ngx_http_headers_in_t;

        fn drop = never_drop::<ffi::ngx_http_headers_in_t>;
    }
}

impl HeadersInRef {
    property!(headers: Headers);

    header!(host);
    header!(connection);
    header!(if_modified_since);
    header!(if_unmodified_since);
    header!(if_match);
    header!(if_none_match);
    header!(user_agent);
    header!(referer);
    header!(content_length);
    header!(content_range);
    header!(content_type);

    header!(range);
    header!(if_range);

    header!(transfer_encoding);
    header!(te);
    header!(expect);
    header!(upgrade);

    cfg_if! {
        if #[cfg(any(feature = "http_gzip", feature = "http_headers"))] {
            header!(accept_encoding);
            header!(via);
        }
    }

    header!(authorization);

    header!(keep_alive);

    #[cfg(feature = "http_x_forwarded_for")]
    header!(x_forwarded_for);

    #[cfg(feature = "http_realip")]
    header!(x_real_ip);

    cfg_if! {
        if #[cfg(feature = "http_headers")] {
            header!(accept);
            header!(accept_language);
        }
    }

    cfg_if! {
        if #[cfg(feature = "http_dav")] {
            header!(depth);
            header!(destination);
            header!(overwrite);
            header!(date);
        }
    }

    header!(cookie);

    str!(user);
    str!(passwd);
    str!(server);
    property!(content_length_n: i64);
    property!(keep_alive_n: i64);
    property!(connection_type() as ConnectionType);
    flag!(chunked());
    flag!(multi());
    flag!(multi_linked());
    flag!(msie());
    flag!(msie6());
    flag!(opera());
    flag!(gecko());
    flag!(chrome());
    flag!(safari());
    flag!(konqueror());
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
    property!(headers: Headers);
    property!(trailers: Headers);

    property!(status: usize);
    property!(status_line: Str);

    header!(server);
    header!(date);
    header!(content_length);
    header!(content_encoding);
    header!(location);
    header!(refresh);
    header!(last_modified);
    header!(content_range);
    header!(accept_ranges);
    header!(www_authenticate);
    header!(expires);
    header!(etag);

    header!(cache_control);
    header!(link);

    pub fn override_charset(&self) -> Option<Str> {
        unsafe { Str::from_ptr(self.as_raw().override_charset) }
    }

    property!(content_type_len: usize);
    property!(content_type: Str);
    property!(charset: Str);

    pub fn content_type_lowcase(&self) -> Option<&CStr> {
        unsafe {
            NonNull::new(self.as_raw().content_type_lowcase)
                .map(|p| CStr::from_ptr(p.as_ptr() as *const _))
        }
    }

    property!(content_type_hash: usize);
    property!(content_length_n: i64);
    property!(content_offset: i64);
    property!(date_time: i64);
    property!(last_modified_time: i64);
}

foreign_type! {
    pub unsafe type Body: Send {
        type CType = ffi::ngx_http_request_body_t;

        fn drop = never_drop::<ffi::ngx_http_request_body_t>;
    }
}

impl BodyRef {
    property!(rest: i64);
    property!(received: i64);
    flag!(filter_need_buffering());
    flag!(last_sent());
    flag!(last_saved());
}
