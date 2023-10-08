use foreign_types::foreign_type;

use crate::{event::PeerConnRef, ffi, never_drop};

foreign_type! {
    pub unsafe type Upstream: Send {
        type CType = ffi::ngx_http_upstream_t;

        fn drop = never_drop::<ffi::ngx_http_upstream_t>;
    }
}

impl UpstreamRef {
    property!(&mut peer: &mut PeerConnRef);
}
