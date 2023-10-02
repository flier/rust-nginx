use foreign_types::foreign_type;

use crate::{core::ModuleRef, ffi, never_drop, AsRawRef};

foreign_type! {
    pub unsafe type Context: Send {
        type CType = ffi::ngx_http_conf_ctx_t;

        fn drop = never_drop::<ffi::ngx_http_conf_ctx_t>;
    }
}

impl ContextRef {
    /// Get the main configuration for the module.
    pub fn main_conf_for<T>(&self, m: &ModuleRef) -> Option<&mut T> {
        unsafe { self.main_conf(m.context_index()) }
    }

    /// Get the server configuration for the module.
    pub fn srv_conf_for<T>(&self, m: &ModuleRef) -> Option<&mut T> {
        unsafe { self.srv_conf(m.context_index()) }
    }

    /// Get the location configuration for the module.
    pub fn loc_conf_for<T>(&self, m: &ModuleRef) -> Option<&mut T> {
        unsafe { self.loc_conf(m.context_index()) }
    }

    /// Get the main configuration from context.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `main_conf` array.
    pub unsafe fn main_conf<T>(&self, idx: usize) -> Option<&mut T> {
        self.as_raw().main_conf.add(idx).cast::<T>().as_mut()
    }

    /// Get the server configuration from context.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `srv_conf` array.
    pub unsafe fn srv_conf<T>(&self, idx: usize) -> Option<&mut T> {
        self.as_raw().srv_conf.add(idx).cast::<T>().as_mut()
    }

    /// Get the location configuration from context.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `loc_conf` array.
    pub unsafe fn loc_conf<T>(&self, idx: usize) -> Option<&mut T> {
        self.as_raw().loc_conf.add(idx).cast::<T>().as_mut()
    }
}
