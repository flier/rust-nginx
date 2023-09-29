use std::{
    ffi::CStr,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use bitflags::bitflags;
use cfg_if::cfg_if;
use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{core::Str, ffi, flag, get, never_drop, str, AsRawMut, AsRawRef};

bitflags! {
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

macro_rules! header {
    ($name:ident) => {
        get!($name as Header);
    };
}

foreign_type! {
    pub unsafe type Request: Send {
        type CType = ffi::ngx_http_request_t;

        fn drop = never_drop::<ffi::ngx_http_request_t>;
    }
}

impl Deref for RequestRef {
    type Target = <Self as ForeignTypeRef>::CType;

    fn deref(&self) -> &Self::Target {
        unsafe { self.as_raw() }
    }
}

impl DerefMut for RequestRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.as_raw_mut() }
    }
}

impl RequestRef {
    get!(headers_in: &HeadersInRef);
    get!(headers_out: &HeadersOutRef);
    get!(request_body as &BodyRef);

    pub fn method(&self) -> Method {
        Method::from_bits_truncate(unsafe { self.as_raw().method as u32 })
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
    get!(main as &Self);
    get!(parent as &Self);
}

foreign_type! {
    pub unsafe type HeadersIn: Send {
        type CType = ffi::ngx_http_headers_in_t;

        fn drop = never_drop::<ffi::ngx_http_headers_in_t>;
    }
}

impl HeadersInRef {
    get!(headers: Headers);

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
    get!(content_length_n: i64);
    get!(keep_alive_n: i64);
    get!(connection_type() as ConnectionType);
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
    get!(headers: Headers);
    get!(trailers: Headers);

    get!(status: usize);
    get!(status_line: Str);

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

    pub fn override_charset(&self) -> Option<&Str> {
        unsafe {
            NonNull::new(self.as_raw().override_charset).and_then(|p| Str::from_ptr(p.as_ptr()))
        }
    }

    get!(content_type_len: usize);
    get!(content_type: Str);
    get!(charset: Str);

    pub fn content_type_lowcase(&self) -> Option<&CStr> {
        unsafe {
            NonNull::new(self.as_raw().content_type_lowcase)
                .map(|p| CStr::from_ptr(p.as_ptr() as *const _))
        }
    }

    get!(content_type_hash: usize);
    get!(content_length_n: i64);
    get!(content_offset: i64);
    get!(date_time: i64);
    get!(last_modified_time: i64);
}

foreign_type! {
    pub unsafe type Body: Send {
        type CType = ffi::ngx_http_request_body_t;

        fn drop = never_drop::<ffi::ngx_http_request_body_t>;
    }
}

impl BodyRef {
    get!(rest: i64);
    get!(received: i64);
    flag!(filter_need_buffering());
    flag!(last_sent());
    flag!(last_saved());
}
