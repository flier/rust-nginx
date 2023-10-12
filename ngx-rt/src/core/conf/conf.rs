use std::{ffi::CStr, ptr::NonNull};

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
    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.as_raw().name) }
    }

    pub fn args(&self) -> &ArrayRef<Str> {
        unsafe { ArrayRef::from_ptr(self.as_raw().args) }
    }

    pub fn cycle(&self) -> &CycleRef {
        unsafe { CycleRef::from_ptr(self.as_raw().cycle) }
    }

    pub fn pool(&self) -> &PoolRef {
        unsafe { PoolRef::from_ptr(self.as_raw().pool) }
    }

    pub fn temp_pool(&self) -> &PoolRef {
        unsafe { PoolRef::from_ptr(self.as_raw().temp_pool) }
    }

    pub fn as_http_context(&self) -> Option<&http::ContextRef> {
        if self.module_type() == ModuleType::Http {
            unsafe {
                NonNull::new(self.as_raw().ctx)
                    .map(|p| http::ContextRef::from_ptr(p.cast().as_ptr()))
            }
        } else {
            None
        }
    }

    pub fn module_type(&self) -> ModuleType {
        ModuleType::from(unsafe { self.as_raw().module_type as u32 })
    }
}
