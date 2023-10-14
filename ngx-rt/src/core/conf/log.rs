use std::ffi::CString;

use foreign_types::ForeignTypeRef;

use crate::{
    core::{ConfRef, LogLevel, LogRef, Logger},
    ffi, AsRawRef,
};

impl ConfRef {
    pub fn log(&self) -> &LogRef {
        unsafe { LogRef::from_ptr(self.as_raw().log) }
    }

    pub fn log_error<S: Into<Vec<u8>>>(&self, level: LogLevel, err: Option<i32>, msg: S) {
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

impl Logger for ConfRef {
    fn core<S: Into<Vec<u8>>>(&self, level: LogLevel, msg: S) {
        self.log_error(level, None, msg)
    }
}

impl<'a> Logger for &'a ConfRef {
    fn core<S: Into<Vec<u8>>>(&self, level: LogLevel, msg: S) {
        self.log_error(level, None, msg)
    }
}
