use std::{ffi::CStr, mem};

use foreign_types::foreign_type;

use crate::{ffi, never_drop, AsRawRef};

foreign_type! {
    pub unsafe type Module: Send {
        type CType = ffi::ngx_module_t;

        fn drop = never_drop::<ffi::ngx_module_t>;
    }
}

impl ModuleRef {
    pub fn context_index(&self) -> usize {
        unsafe { self.as_raw().ctx_index }
    }

    pub fn index(&self) -> usize {
        unsafe { self.as_raw().index }
    }

    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.as_raw().name) }
    }

    pub fn ty(&self) -> Type {
        unsafe { mem::transmute(self.as_raw().type_ as u32) }
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Core = ffi::NGX_CORE_MODULE,
    Conf = ffi::NGX_CONF_MODULE,
    #[cfg(feature = "event")]
    Event = ffi::NGX_EVENT_MODULE,
    #[cfg(feature = "http")]
    Http = ffi::NGX_HTTP_MODULE,
    #[cfg(feature = "mail")]
    Mail = ffi::NGX_MAIL_MODULE,

    Stream = ffi::NGX_STREAM_MODULE,
}
