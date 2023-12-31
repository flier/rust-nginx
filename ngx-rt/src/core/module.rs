use std::slice;

use foreign_types::foreign_type;
use num_enum::FromPrimitive;

use crate::{core::Cmds, ffi, never_drop, property, AsRawRef};

foreign_type! {
    pub unsafe type Module: Send {
        type CType = ffi::ngx_module_t;

        fn drop = never_drop::<ffi::ngx_module_t>;
    }
}

impl ModuleRef {
    property! {
        ctx_index: usize;
        index: usize;
        version: usize;
        name: &CStr;
        signature: &CStr;
    }

    pub fn ty(&self) -> Type {
        Type::from(unsafe { self.as_raw().type_ as u32 })
    }

    pub fn commands(&self) -> Cmds {
        unsafe {
            let r = self.as_raw();
            let len = (0..usize::MAX)
                .find(|&n| {
                    r.commands
                        .add(n)
                        .as_ref()
                        .map_or(true, |cmd| cmd.name.len == 0)
                })
                .unwrap_or(0);

            Cmds::from(slice::from_raw_parts(r.commands, len))
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, FromPrimitive)]
pub enum Type {
    Core = ffi::NGX_CORE_MODULE,
    Conf = ffi::NGX_CONF_MODULE,
    #[cfg(feature = "event")]
    Event = ffi::NGX_EVENT_MODULE,
    #[default]
    #[cfg(feature = "http")]
    Http = ffi::NGX_HTTP_MODULE,
    #[cfg(feature = "mail")]
    Mail = ffi::NGX_MAIL_MODULE,
    #[cfg(feature = "stream")]
    Stream = ffi::NGX_STREAM_MODULE,
}
