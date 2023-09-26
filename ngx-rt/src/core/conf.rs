use std::{
    ffi::{c_char, CStr, CString},
    ptr::{self, NonNull},
};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{fake_drop, ffi, http, AsRaw};

use super::{log, ArrayRef, BufRef, CycleRef, LogRef, ModuleType, PoolRef, Str};

pub const NGX_CONF_OK: *mut c_char = ptr::null_mut();
pub const NGX_CONF_ERROR: *mut c_char = usize::MAX as *mut c_char;

foreign_type! {
    pub unsafe type Conf: Send {
        type CType = ffi::ngx_conf_t;

        fn drop = fake_drop::<ffi::ngx_conf_t>;
    }
}

impl ConfRef {
    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.as_raw().name) }
    }

    pub fn args(&self) -> &ArrayRef<&Str> {
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

    pub fn conf_file(&self) -> &ConfFileRef {
        unsafe { ConfFileRef::from_ptr(self.as_raw().conf_file) }
    }

    pub fn log(&self) -> &LogRef {
        unsafe { LogRef::from_ptr(self.as_raw().log) }
    }

    pub fn stderr<S: Into<Vec<u8>>>(&self, msg: S) {
        self.log_error(log::Level::STDERR, None, msg)
    }

    pub fn emerg<S: Into<Vec<u8>>>(&self, msg: S) {
        self.log_error(log::Level::EMERG, None, msg)
    }

    pub fn alert<S: Into<Vec<u8>>>(&self, msg: S) {
        self.log_error(log::Level::ALERT, None, msg)
    }

    pub fn critical<S: Into<Vec<u8>>>(&self, msg: S) {
        self.log_error(log::Level::CRIT, None, msg)
    }

    pub fn error<S: Into<Vec<u8>>>(&self, msg: S) {
        self.log_error(log::Level::EMERG, None, msg)
    }

    pub fn warn<S: Into<Vec<u8>>>(&self, msg: S) {
        self.log_error(log::Level::WARN, None, msg)
    }

    pub fn notice<S: Into<Vec<u8>>>(&self, msg: S) {
        self.log_error(log::Level::NOTICE, None, msg)
    }

    pub fn info<S: Into<Vec<u8>>>(&self, msg: S) {
        self.log_error(log::Level::NOTICE, None, msg)
    }

    pub fn debug<S: Into<Vec<u8>>>(&self, msg: S) {
        self.log_error(log::Level::DEBUG, None, msg)
    }

    pub fn log_error<S: Into<Vec<u8>>>(&self, level: log::Level, err: Option<i32>, msg: S) {
        let msg = CString::new(msg).expect("msg");

        unsafe {
            ffi::ngx_conf_log_error(
                level.bits() as usize,
                self.as_ptr(),
                err.unwrap_or_default(),
                msg.as_ptr(),
            );
        }
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
        ModuleType::try_from(unsafe { self.as_raw().module_type as u32 }).expect("module_type")
    }
}

foreign_type! {
    pub unsafe type ConfFile: Send {
        type CType = ffi::ngx_conf_file_t;

        fn drop = fake_drop::<ffi::ngx_conf_file_t>;
    }
}

impl ConfFileRef {
    pub fn buffer(&self) -> &BufRef {
        unsafe { BufRef::from_ptr(self.as_raw().buffer) }
    }

    pub fn dump(&self) -> &BufRef {
        unsafe { BufRef::from_ptr(self.as_raw().dump) }
    }

    pub fn line(&self) -> usize {
        unsafe { self.as_raw().line }
    }
}
