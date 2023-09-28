use std::ffi::CString;

use foreign_types::ForeignTypeRef;

use crate::{
    core::{log, ConfRef, LogRef},
    ffi, AsRawRef,
};

impl ConfRef {
    pub fn log(&self) -> &LogRef {
        unsafe { LogRef::from_ptr(self.as_raw().log) }
    }

    pub fn stderr<S: Into<Vec<u8>>>(&self, msg: S) {
        self.log_error(log::Level::StdErr, None, msg)
    }

    pub fn emerg<S: Into<Vec<u8>>>(&self, msg: S) {
        self.log_error(log::Level::Emerg, None, msg)
    }

    pub fn alert<S: Into<Vec<u8>>>(&self, msg: S) {
        self.log_error(log::Level::Alert, None, msg)
    }

    pub fn critical<S: Into<Vec<u8>>>(&self, msg: S) {
        self.log_error(log::Level::Critical, None, msg)
    }

    pub fn error<S: Into<Vec<u8>>>(&self, msg: S) {
        self.log_error(log::Level::Error, None, msg)
    }

    pub fn warn<S: Into<Vec<u8>>>(&self, msg: S) {
        self.log_error(log::Level::Warn, None, msg)
    }

    pub fn notice<S: Into<Vec<u8>>>(&self, msg: S) {
        self.log_error(log::Level::Notice, None, msg)
    }

    pub fn info<S: Into<Vec<u8>>>(&self, msg: S) {
        self.log_error(log::Level::Info, None, msg)
    }

    pub fn debug<S: Into<Vec<u8>>>(&self, msg: S) {
        self.log_error(log::Level::Debug, None, msg)
    }

    pub fn log_error<S: Into<Vec<u8>>>(&self, level: log::Level, err: Option<i32>, msg: S) {
        let msg = CString::new(msg).expect("msg");

        unsafe {
            ffi::ngx_conf_log_error(
                level as usize,
                self.as_ptr(),
                err.unwrap_or_default(),
                msg.as_ptr(),
            );
        }
    }
}
