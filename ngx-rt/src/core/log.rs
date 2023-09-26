use std::ffi::CStr;

use bitflags::bitflags;
use foreign_types::{foreign_type, ForeignTypeRef};

use crate::ffi;

use super::fake_drop;

foreign_type! {
    pub unsafe type Log: Send {
        type CType = ffi::ngx_log_t;

        fn drop = fake_drop::<ffi::ngx_log_t>;
    }
}

impl LogRef {
    pub fn level(&self) -> LogLevel {
        LogLevel::from_bits_truncate(unsafe { self.as_raw().log_level })
    }

    #[cfg(feature = "debug_log")]
    pub fn debug_core(&self, err: Option<i32>, msg: &CStr) {
        unsafe { ffi::ngx_log_debug_core(self.as_ptr(), err.unwrap_or_default(), msg.as_ptr()) }
    }

    pub fn error_core(&self, level: LogLevel, err: Option<i32>, msg: &CStr) {
        if self.level().contains(level) {
            unsafe {
                ffi::ngx_log_error_core(
                    level.bits(),
                    self.as_ptr(),
                    err.unwrap_or_default(),
                    msg.as_ptr(),
                )
            }
        }
    }

    unsafe fn as_raw(&self) -> &ffi::ngx_log_t {
        &*self.as_ptr()
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct LogLevel: usize {
        const CORE = 0x010;
        const ALLOC = 0x020;
        const MUTEX = 0x040;
        const EVENT = 0x080;
        const HTTP = 0x100;
        const MAIL = 0x200;
        const STREAM = 0x400;
    }
}
