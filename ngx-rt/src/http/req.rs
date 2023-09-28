use std::ops::{Deref, DerefMut};

use bitflags::bitflags;
use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{
    core::{hash, Str},
    ffi, never_drop, AsRawMut, AsRawRef, FromRawRef,
};

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
    pub fn headers_in(&self) -> &HeadersInRef {
        unsafe { HeadersInRef::from_ptr(&self.as_raw().headers_in as *const _ as *mut _) }
    }

    pub fn headers_out(&self) -> &HeadersOutRef {
        unsafe { HeadersOutRef::from_ptr(&self.as_raw().headers_out as *const _ as *mut _) }
    }

    pub fn body(&self) -> Option<&BodyRef> {
        unsafe { BodyRef::from_raw(self.as_raw().request_body) }
    }

    pub fn method(&self) -> Method {
        Method::from_bits_truncate(unsafe { self.as_raw().method as u32 })
    }

    pub fn version(&self) -> (u32, u32) {
        unsafe {
            let v = self.as_raw().http_version;

            ((v >> 16) as u32, (v & 0xFFFF) as u32)
        }
    }

    pub fn request_line(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw().request_line) }
    }

    pub fn uri(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw().uri) }
    }

    pub fn args(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw().args) }
    }

    pub fn exten(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw().exten) }
    }

    pub fn unparsed_uri(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw().unparsed_uri) }
    }

    pub fn method_name(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw().method_name) }
    }

    pub fn http_protocol(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw().http_protocol) }
    }

    pub fn schema(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw().schema) }
    }

    pub fn main(&self) -> Option<&Self> {
        unsafe { Self::from_raw(self.as_raw().main) }
    }

    pub fn parent(&self) -> Option<&Self> {
        unsafe { Self::from_raw(self.as_raw().parent) }
    }
}

foreign_type! {
    pub unsafe type HeadersIn: Send {
        type CType = ffi::ngx_http_headers_in_t;

        fn drop = never_drop::<ffi::ngx_http_headers_in_t>;
    }
}

macro_rules! header {
    ($name:ident) => {
        pub fn $name(&self) -> Option<&hash::TableEltRef> {
            unsafe { hash::TableEltRef::from_raw(self.as_raw().$name) }
        }
    };
}

impl HeadersInRef {
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

    cfg_if::cfg_if! {
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

    #[cfg(feature = "http_headers")]
    header!(accept);
    #[cfg(feature = "http_headers")]
    header!(accept_language);

    cfg_if::cfg_if! {
        if #[cfg(feature = "http_dav")] {
            header!(depth);
            header!(destination);
            header!(overwrite);
            header!(date);
        }
    }

    header!(cookie);
}

foreign_type! {
    pub unsafe type HeadersOut: Send {
        type CType = ffi::ngx_http_headers_out_t;

        fn drop = never_drop::<ffi::ngx_http_headers_out_t>;
    }
}

foreign_type! {
    pub unsafe type Body: Send {
        type CType = ffi::ngx_http_request_body_t;

        fn drop = never_drop::<ffi::ngx_http_request_body_t>;
    }
}
