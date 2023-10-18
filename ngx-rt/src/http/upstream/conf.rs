use std::ptr::NonNull;

use foreign_types::foreign_type;

use crate::{ffi, http::UnsafeSrvConf, never_drop, property, AsRawRef};

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

impl UnsafeSrvConf for SrvConfRef {
    unsafe fn unchecked_srv_conf<T>(&self, idx: usize) -> Option<NonNull<T>> {
        NonNull::new(self.as_raw().srv_conf.add(idx).read().cast())
    }
}

impl SrvConfRef {
    property!(&mut peer: &mut PeerRef);
}
