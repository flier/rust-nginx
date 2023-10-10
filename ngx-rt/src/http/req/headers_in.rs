use foreign_types::foreign_type;
use num_enum::FromPrimitive;

use crate::{raw::never_drop, AsRawRef};

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
        content_length_n: i64;
        headers: Headers;
        keep_alive_n: i64;
    }

    flag! {
        chunked;
        multi;
        multi_linked;
        msie;
        msie6;
        opera;
        gecko;
        chrome;
        safari;
        konqueror;
    }

    pub fn connection_type(&self) -> ConnType {
        ConnType::from(unsafe { self.as_raw().connection_type() })
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, FromPrimitive)]
pub enum ConnType {
    #[default]
    Close = ffi::NGX_HTTP_CONNECTION_CLOSE,
    KeepAlive = ffi::NGX_HTTP_CONNECTION_KEEP_ALIVE,
}
