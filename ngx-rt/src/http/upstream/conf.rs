use foreign_types::foreign_type;

use crate::{ffi, never_drop, property};

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
    property!(peer: &mut PeerRef);
}
