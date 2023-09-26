use foreign_types::foreign_type;

use crate::{fake_drop, ffi, AsRaw};

foreign_type! {
    pub unsafe type Context: Send {
        type CType = ffi::ngx_http_conf_ctx_t;

        fn drop = fake_drop::<ffi::ngx_http_conf_ctx_t>;
    }
}

impl ContextRef {
    pub unsafe fn main_conf<T>(&self, idx: usize) -> Option<&mut T> {
        self.as_raw().main_conf.add(idx).cast::<T>().as_mut()
    }

    pub unsafe fn srv_conf<T>(&self, idx: usize) -> Option<&mut T> {
        self.as_raw().srv_conf.add(idx).cast::<T>().as_mut()
    }

    pub unsafe fn loc_conf<T>(&self, idx: usize) -> Option<&mut T> {
        self.as_raw().loc_conf.add(idx).cast::<T>().as_mut()
    }
}
