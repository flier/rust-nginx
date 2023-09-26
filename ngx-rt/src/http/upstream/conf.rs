use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{fake_drop, ffi, AsRaw};

use super::PeerRef;

foreign_type! {
    pub unsafe type MainConf: Send {
        type CType = ffi::ngx_http_upstream_main_conf_t;

        fn drop = fake_drop::<ffi::ngx_http_upstream_main_conf_t>;
    }
}

foreign_type! {
    pub unsafe type SrvConf: Send {
        type CType = ffi::ngx_http_upstream_srv_conf_t;

        fn drop = fake_drop::<ffi::ngx_http_upstream_srv_conf_t>;
    }
}

impl SrvConfRef {
    pub fn peer(&self) -> &PeerRef {
        unsafe { PeerRef::from_ptr(&self.as_raw().peer as *const _ as *mut _) }
    }

    pub fn peer_mut(&mut self) -> &mut PeerRef {
        unsafe { PeerRef::from_ptr_mut(&mut self.as_raw_mut().peer) }
    }
}
