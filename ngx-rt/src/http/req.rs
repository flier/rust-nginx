use std::ops::{Deref, DerefMut};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{ffi, never_drop, AsRaw};

foreign_type! {
    pub unsafe type Request: Send {
        type CType = ffi::ngx_http_request_t;

        fn drop = never_drop::<ffi::ngx_http_request_t>;
    }
}

impl Deref for RequestRef {
    type Target = <Self as ForeignTypeRef>::CType;

    fn deref(&self) -> &Self::Target {
        unsafe { self.as_raw() }
    }
}

impl DerefMut for RequestRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.as_raw_mut() }
    }
}
