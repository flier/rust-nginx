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
    property!(peer: &mut PeerRef);

    /// Get the reference of server configuration for the module.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the module is initialized.
    pub fn srv_conf<T>(&self, m: &ModuleRef) -> Option<&T> {
        unsafe {
            self.as_raw()
                .srv_conf
                .add(m.context_index())
                .cast::<T>()
                .as_ref()
        }
    }

    /// Get the mutable reference of server configuration for the module.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the module is initialized.
    pub fn srv_conf_mut<T>(&mut self, m: &ModuleRef) -> Option<&mut T> {
        unsafe {
            self.as_raw()
                .srv_conf
                .add(m.context_index())
                .cast::<T>()
                .as_mut()
        }
    }
}
