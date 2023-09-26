use std::ops::{Deref, DerefMut};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{fake_drop, ffi, AsRaw};

foreign_type! {
    pub unsafe type Peer: Send {
        type CType = ffi::ngx_http_upstream_peer_t;

        fn drop = fake_drop::<ffi::ngx_http_upstream_peer_t>;
    }
}

impl Deref for PeerRef {
    type Target = <Self as ForeignTypeRef>::CType;

    fn deref(&self) -> &Self::Target {
        unsafe { self.as_raw() }
    }
}

impl DerefMut for PeerRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.as_raw_mut() }
    }
}
