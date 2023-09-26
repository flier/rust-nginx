use std::ffi::CString;

use bitflags::bitflags;
use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{fake_drop, ffi, AsRaw};

foreign_type! {
    pub unsafe type Log: Send {
        type CType = ffi::ngx_log_t;

        fn drop = fake_drop::<ffi::ngx_log_t>;
    }
}

impl LogRef {
    pub fn core(&self) -> WithModule {
        WithModule(self, Level::CORE)
    }

    pub fn alloc(&self) -> WithModule {
        WithModule(self, Level::ALLOC)
    }

    pub fn mutex(&self) -> WithModule {
        WithModule(self, Level::MUTEX)
    }

    pub fn event(&self) -> WithModule {
        WithModule(self, Level::EVENT)
    }

    pub fn http(&self) -> WithModule {
        WithModule(self, Level::HTTP)
    }

    pub fn mail(&self) -> WithModule {
        WithModule(self, Level::MAIL)
    }

    pub fn stream(&self) -> WithModule {
        WithModule(self, Level::STREAM)
    }

    pub fn level(&self) -> Level {
        Level::from_bits_truncate(unsafe { self.as_raw().log_level as u32 })
    }

    #[cfg(feature = "debug_log")]
    pub fn debug_core(&self, err: Option<i32>, msg: &CStr) {
        unsafe { ffi::ngx_log_debug_core(self.as_ptr(), err.unwrap_or_default(), msg.as_ptr()) }
    }

    pub fn error_core<S: Into<Vec<u8>>>(&self, level: Level, err: Option<i32>, msg: S) {
        if self.level() >= level {
            let msg = CString::new(msg).expect("msg");

            unsafe {
                ffi::ngx_log_error_core(
                    level.bits() as usize,
                    self.as_ptr(),
                    err.unwrap_or_default(),
                    msg.as_ptr(),
                )
            }
        }
    }
}

pub struct WithModule<'a>(&'a LogRef, Level);

impl<'a> WithModule<'a> {
    pub fn stderr<S: Into<Vec<u8>>>(&self, msg: S) {
        if self.0.level().contains(self.1) {
            self.log(Level::STDERR, msg)
        }
    }

    pub fn emerg<S: Into<Vec<u8>>>(&self, msg: S) {
        if self.0.level().contains(self.1) {
            self.log(Level::EMERG, msg)
        }
    }

    pub fn alert<S: Into<Vec<u8>>>(&self, msg: S) {
        if self.0.level().contains(self.1) {
            self.log(Level::ALERT, msg)
        }
    }

    pub fn critical<S: Into<Vec<u8>>>(&self, msg: S) {
        if self.0.level().contains(self.1) {
            self.log(Level::CRIT, msg)
        }
    }

    pub fn error<S: Into<Vec<u8>>>(&self, msg: S) {
        if self.0.level().contains(self.1) {
            self.log(Level::ERR, msg)
        }
    }

    pub fn warn<S: Into<Vec<u8>>>(&self, msg: S) {
        if self.0.level().contains(self.1) {
            self.log(Level::WARN, msg)
        }
    }

    pub fn notice<S: Into<Vec<u8>>>(&self, msg: S) {
        if self.0.level().contains(self.1) {
            self.log(Level::NOTICE, msg)
        }
    }

    pub fn debug<S: Into<Vec<u8>>>(&self, msg: S) {
        if self.0.level().contains(self.1) {
            self.log(Level::DEBUG, msg)
        }
    }

    pub fn log<S: Into<Vec<u8>>>(&self, level: Level, msg: S) {
        if self.0.level().contains(self.1) {
            self.0.error_core(level, None, msg);
        }
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Level: u32 {
        const STDERR = ffi::NGX_LOG_STDERR;
        const EMERG = ffi::NGX_LOG_EMERG;
        const ALERT = ffi::NGX_LOG_ALERT;
        const CRIT = ffi::NGX_LOG_CRIT;
        const ERR = ffi::NGX_LOG_ERR;
        const WARN = ffi::NGX_LOG_WARN;
        const NOTICE = ffi::NGX_LOG_NOTICE;
        const INFO = ffi::NGX_LOG_INFO;
        const DEBUG = ffi::NGX_LOG_DEBUG;

        const CORE = ffi::NGX_LOG_DEBUG_CORE;
        const ALLOC = ffi::NGX_LOG_DEBUG_ALLOC;
        const MUTEX = ffi::NGX_LOG_DEBUG_MUTEX;
        const EVENT = ffi::NGX_LOG_DEBUG_EVENT;
        const HTTP = ffi::NGX_LOG_DEBUG_HTTP;
        const MAIL = ffi::NGX_LOG_DEBUG_MAIL;
        const STREAM = ffi::NGX_LOG_DEBUG_STREAM;
    }
}
