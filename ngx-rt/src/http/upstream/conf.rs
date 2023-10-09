use std::ptr::NonNull;

use foreign_types::foreign_type;

use crate::{core::ModuleRef, ffi, never_drop, property, AsRawRef};

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
    property!(&mut peer: &mut PeerRef);

    /// Get the reference of server configuration for the module.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the module is initialized.
    pub fn srv_conf_for<T>(&self, m: &ModuleRef) -> Option<&mut T> {
        unsafe { self.srv_conf(m.context_index()) }
    }

    /// Get the server configuration from context.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `srv_conf` array.
    pub unsafe fn srv_conf<T>(&self, idx: usize) -> Option<&mut T> {
        NonNull::new(self.as_raw().srv_conf.add(idx).read()).map(|p| p.cast::<T>().as_mut())
    }
}
