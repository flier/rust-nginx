use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{ffi, never_drop, AsRawRef};

use super::{conf::OpenFile, shm, ListRef, LogRef, ModuleRef, PoolRef, Str};

foreign_type! {
    pub unsafe type Cycle: Send {
        type CType = ffi::ngx_cycle_t;

        fn drop = never_drop::<ffi::ngx_cycle_t>;
    }
}

impl CycleRef {
    pub fn pool(&self) -> &PoolRef {
        unsafe { PoolRef::from_ptr(self.as_raw_ref().pool) }
    }

    pub fn log(&self) -> &LogRef {
        unsafe { LogRef::from_ptr(self.as_raw_ref().log) }
    }

    pub fn modules(&self) -> &[&ModuleRef] {
        unsafe {
            let r = self.as_raw_ref();

            std::slice::from_raw_parts(r.modules.cast(), r.modules_n)
        }
    }

    pub fn open_files(&self) -> &ListRef<OpenFile> {
        unsafe { ListRef::from_ptr(&self.as_raw_ref().open_files as *const _ as *mut _) }
    }

    pub fn shared_memory(&self) -> &ListRef<shm::Zone> {
        unsafe { ListRef::from_ptr(&self.as_raw_ref().shared_memory as *const _ as *mut _) }
    }

    pub fn conf_file(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw_ref().conf_file) }
    }

    pub fn conf_param(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw_ref().conf_param) }
    }

    pub fn conf_prefix(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw_ref().conf_prefix) }
    }

    pub fn prefix(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw_ref().prefix) }
    }

    pub fn error_log(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw_ref().error_log) }
    }

    pub fn lock_file(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw_ref().lock_file) }
    }

    pub fn hostname(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw_ref().hostname) }
    }
}
