use std::ptr::NonNull;

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{
    core::{ArrayRef, CycleRef, ModuleType, PoolRef, Str},
    ffi, http, never_drop, AsRawRef,
};

foreign_type! {
    pub unsafe type Conf: Send {
        type CType = ffi::ngx_conf_t;

        fn drop = never_drop::<ffi::ngx_conf_t>;
    }
}

impl ConfRef {
    property! {
        cycle: &CycleRef;
        pool: &PoolRef;
        temp_pool: &PoolRef;
        name: &CStr;
        args: &ArrayRef<Str>;
    }

    pub fn as_http_context(&self) -> Option<&http::ConfContextRef> {
        if self.module_type() == ModuleType::Http {
            unsafe {
                NonNull::new(self.as_raw().ctx)
                    .map(|p| http::ConfContextRef::from_ptr(p.cast().as_ptr()))
            }
        } else {
            None
        }
    }

    pub fn module_type(&self) -> ModuleType {
        ModuleType::from(unsafe { self.as_raw().module_type as u32 })
    }
}
