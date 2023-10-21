use foreign_types::foreign_type;

use crate::{ffi, flag, never_drop, property, str};

foreign_type! {
    pub unsafe type SrvConf: Send {
        type CType = ffi::ngx_http_core_srv_conf_t;

        fn drop = never_drop::<ffi::ngx_http_core_srv_conf_t>;
    }
}

impl SrvConfRef {
    str! {
        &server_name?;
    }

    property! {
        connection_pool_size: usize;
        request_pool_size: usize;
        client_header_buffer_size: usize;
    }

    flag! {
        listen;
    }
}
