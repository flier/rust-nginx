use foreign_types::foreign_type;

use crate::{ffi, get, never_drop};

use super::PeerRef;

foreign_type! {
    pub unsafe type MainConf: Send {
        type CType = ffi::ngx_http_upstream_main_conf_t;

        fn drop = never_drop::<ffi::ngx_http_upstream_main_conf_t>;
    }
}

foreign_type! {
    pub unsafe type SrvConf: Send {
        type CType = ffi::ngx_http_upstream_srv_conf_t;

        fn drop = never_drop::<ffi::ngx_http_upstream_srv_conf_t>;
    }
}

impl SrvConfRef {
    get!(peer: &PeerRef);
}
