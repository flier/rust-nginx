use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{event::PeerConnRef, ffi, never_drop, AsRawMut, AsRawRef};

foreign_type! {
    pub unsafe type Upstream: Send {
        type CType = ffi::ngx_http_upstream_t;

        fn drop = never_drop::<ffi::ngx_http_upstream_t>;
    }
}

impl UpstreamRef {
    pub fn peer(&self) -> &PeerConnRef {
        unsafe { PeerConnRef::from_ptr(&self.as_raw().peer as *const _ as *mut _) }
    }

    pub fn peer_mut(&mut self) -> &mut PeerConnRef {
        unsafe { PeerConnRef::from_ptr_mut(&mut self.as_raw_mut().peer as *mut _) }
    }
}
