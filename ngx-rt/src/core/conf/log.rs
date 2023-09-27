use std::ffi::CString;

use foreign_types::ForeignTypeRef;

use crate::{
    core::{log, ConfRef, LogRef},
    ffi, AsRaw,
};

impl ConfRef {
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
}
