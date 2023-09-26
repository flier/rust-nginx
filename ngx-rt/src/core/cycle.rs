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

    pub fn conf_file(&self) -> Option<&str> {
        unsafe { Str::from_raw(self.as_raw().conf_file) }
            .as_str()
            .expect("conf_file")
    }

    pub fn conf_param(&self) -> Option<&str> {
        unsafe { Str::from_raw(self.as_raw().conf_param) }
            .as_str()
            .expect("conf_param")
    }

    pub fn conf_prefix(&self) -> Option<&str> {
        unsafe { Str::from_raw(self.as_raw().conf_prefix) }
            .as_str()
            .expect("conf_prefix")
    }

    pub fn prefix(&self) -> Option<&str> {
        unsafe { Str::from_raw(self.as_raw().prefix) }
            .as_str()
            .expect("prefix")
    }

    pub fn error_log(&self) -> Option<&str> {
        unsafe { Str::from_raw(self.as_raw().error_log) }
            .as_str()
            .expect("error_log")
    }

    pub fn lock_file(&self) -> Option<&str> {
        unsafe { Str::from_raw(self.as_raw().lock_file) }
            .as_str()
            .expect("lock_file")
    }

    pub fn hostname(&self) -> Option<&str> {
        unsafe { Str::from_raw(self.as_raw().hostname) }
            .as_str()
            .expect("hostname")
    }
}
