use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{fake_drop, ffi, AsRaw};

use super::{LogRef, PoolRef, Str};

foreign_type! {
    pub unsafe type Cycle: Send {
        type CType = ffi::ngx_cycle_t;

        fn drop = fake_drop::<ffi::ngx_cycle_t>;
    }
}

impl CycleRef {
    pub fn pool(&self) -> &PoolRef {
        unsafe { PoolRef::from_ptr(self.as_raw().pool) }
    }

    pub fn log(&self) -> &LogRef {
        unsafe { LogRef::from_ptr(self.as_raw().log) }
    }

    pub fn conf_file(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw().conf_file) }
    }

    pub fn conf_param(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw().conf_param) }
    }

    pub fn conf_prefix(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw().conf_prefix) }
    }

    pub fn prefix(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw().prefix) }
    }

    pub fn error_log(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw().error_log) }
    }

    pub fn lock_file(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw().lock_file) }
    }

    pub fn hostname(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw().hostname) }
    }
}
