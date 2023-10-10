use foreign_types::foreign_type;

use crate::{ffi, flag, never_drop, property};

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
        filter_need_buffering;
        last_sent;
        last_saved;
    }
}
