use foreign_types::foreign_type;

use crate::{ffi, never_drop, AsRawRef};

foreign_type! {
    pub unsafe type Context: Send {
        type CType = ffi::ngx_http_conf_ctx_t;

        fn drop = never_drop::<ffi::ngx_http_conf_ctx_t>;
    }
}

impl ContextRef {
    /// Get the main configuration from context.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `main_conf` array.
    pub unsafe fn main_conf<T>(&self, idx: usize) -> Option<&mut T> {
        self.as_raw_ref().main_conf.add(idx).cast::<T>().as_mut()
    }

    /// Get the server configuration from context.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `srv_conf` array.
    pub unsafe fn srv_conf<T>(&self, idx: usize) -> Option<&mut T> {
        self.as_raw_ref().srv_conf.add(idx).cast::<T>().as_mut()
    }

    /// Get the location configuration from context.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `loc_conf` array.
    pub unsafe fn loc_conf<T>(&self, idx: usize) -> Option<&mut T> {
        self.as_raw_ref().loc_conf.add(idx).cast::<T>().as_mut()
    }
}
