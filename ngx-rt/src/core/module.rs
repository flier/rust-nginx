use std::ffi::CStr;

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
        unsafe { self.as_raw_ref().ctx_index }
    }

    pub fn index(&self) -> usize {
        unsafe { self.as_raw_ref().index }
    }

    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.as_raw_ref().name) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Core,
    Conf,
    Event,
    Http,
    Mail,
    Stream,
}

impl TryFrom<u32> for Type {
    type Error = u32;

    fn try_from(value: u32) -> Result<Type, Self::Error> {
        match value {
            ffi::NGX_CORE_MODULE => Ok(Type::Core),
            ffi::NGX_CONF_MODULE => Ok(Type::Conf),
            #[cfg(feature = "event")]
            ffi::NGX_EVENT_MODULE => Ok(Type::Event),
            #[cfg(feature = "http")]
            ffi::NGX_HTTP_MODULE => Ok(Type::Http),
            #[cfg(feature = "mail")]
            ffi::NGX_MAIL_MODULE => Ok(Type::Mail),
            #[cfg(feature = "stream")]
            ffi::NGX_STREAM_MODULE => Ok(Type::Stream),

            _ => Err(value),
        }
    }
}
