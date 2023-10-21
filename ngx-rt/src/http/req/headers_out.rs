use std::{ffi::CStr, ptr::NonNull};

use foreign_types::foreign_type;

use crate::{raw::never_drop, AsRawRef};

foreign_type! {
    pub unsafe type HeadersOut: Send {
        type CType = ffi::ngx_http_headers_out_t;

        fn drop = never_drop::<ffi::ngx_http_headers_out_t>;
    }
}

impl HeadersOutRef {
    str! {
        &status_line;

        override_charset?;
        &content_type?;
        &charset?;
    }

    property! {
        headers: Headers;
        trailers: Headers;

        status: usize;

        content_type_len: usize;

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

    pub fn content_type_lowcase(&self) -> Option<&CStr> {
        unsafe {
            NonNull::new(self.as_raw().content_type_lowcase)
                .map(|p| CStr::from_ptr(p.as_ptr() as *const _))
        }
    }
}
