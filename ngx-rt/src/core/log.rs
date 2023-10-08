use std::{ffi::CString, mem, path::Path, ptr};

use bitflags::bitflags;
use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{core::time, ffi, never_drop, AsRawMut, AsRawRef, Error, FromRawMut, Result};

use super::conf::OpenFileRef;

foreign_type! {
    pub unsafe type Log: Send {
        type CType = ffi::ngx_log_t;

        fn drop = never_drop::<ffi::ngx_log_t>;
    }
}

impl Log {
    pub fn stderr() -> &'static mut LogRef {
        time::init();

        unsafe {
            LogRef::from_ptr_mut(ffi::ngx_log_init(ptr::null_mut(), b"\0".as_ptr() as *mut _))
        }
    }

    pub fn init(
        prefix: Option<&'_ Path>,
        error_log: Option<&'_ str>,
    ) -> Result<&'static mut LogRef> {
        time::init();

        unsafe {
            LogRef::from_raw_mut(ffi::ngx_log_init(
                prefix
                    .map(|p| CString::new(p.to_string_lossy().to_string()))
                    .transpose()?
                    .as_ref()
                    .map(|s| s.as_ptr() as *mut _)
                    .unwrap_or_else(ptr::null_mut),
                error_log
                    .map(CString::new)
                    .transpose()?
                    .as_ref()
                    .map(|s| s.as_ptr() as *mut _)
                    .unwrap_or_else(ptr::null_mut),
            ))
            .ok_or(Error::OutOfMemory)
        }
    }
}

impl LogRef {
    const LOG_LEVEL_MASK: u32 = 0x000F;
    const LOG_MODULE_MASK: u32 = 0xFFF0;

    pub fn level(&self) -> Level {
        unsafe {
            let level = self.as_raw().log_level as u32;

            mem::transmute(level & Self::LOG_LEVEL_MASK)
        }
    }

    pub fn module(&self) -> Module {
        Module::from_bits_truncate(unsafe {
            self.as_raw().log_level as u32 & Self::LOG_MODULE_MASK
        })
    }

    pub fn with_module(&mut self, module: Module) {
        unsafe { self.as_raw_mut().log_level |= module.bits() as usize }
    }

    pub fn file(&self) -> &OpenFileRef {
        unsafe { OpenFileRef::from_ptr(self.as_raw().file) }
    }

    pub fn core(&self) -> WithModule {
        WithModule(self, Module::CORE)
    }

    pub fn alloc(&self) -> WithModule {
        WithModule(self, Module::ALLOC)
    }

    pub fn mutex(&self) -> WithModule {
        WithModule(self, Module::MUTEX)
    }

    pub fn event(&self) -> WithModule {
        WithModule(self, Module::EVENT)
    }

    pub fn http(&self) -> WithModule {
        WithModule(self, Module::HTTP)
    }

    pub fn mail(&self) -> WithModule {
        WithModule(self, Module::MAIL)
    }

    pub fn stream(&self) -> WithModule {
        WithModule(self, Module::STREAM)
    }

    #[cfg(feature = "debug_log")]
    pub fn debug_core(&self, err: Option<i32>, fmt: &CStr, msg: &CStr) {
        unsafe {
            ffi::ngx_log_debug_core(
                self.as_ptr(),
                err.unwrap_or_default(),
                fmt.as_ptr(),
                msg.as_ptr(),
            )
        }
    }

    pub unsafe fn error_core<S: Into<Vec<u8>>>(&self, level: Level, err: Option<i32>, msg: S) {
        if self.as_raw().log_level >= level as usize {
            let msg = CString::new(msg).expect("msg");

            ffi::ngx_log_error_core(
                level as usize,
                self.as_ptr(),
                err.unwrap_or_default(),
                msg.as_ptr(),
            )
        }
    }
}

pub struct WithModule<'a>(&'a LogRef, Module);

impl<'a> WithModule<'a> {
    pub fn stderr<S: Into<Vec<u8>>>(&self, msg: S) {
        if self.0.module().contains(self.1) {
            self.log(Level::StdErr, msg)
        }
    }

    pub fn emerg<S: Into<Vec<u8>>>(&self, msg: S) {
        if self.0.module().contains(self.1) {
            self.log(Level::Emerg, msg)
        }
    }

    pub fn alert<S: Into<Vec<u8>>>(&self, msg: S) {
        if self.0.module().contains(self.1) {
            self.log(Level::Alert, msg)
        }
    }

    pub fn critical<S: Into<Vec<u8>>>(&self, msg: S) {
        if self.0.module().contains(self.1) {
            self.log(Level::Critical, msg)
        }
    }

    pub fn error<S: Into<Vec<u8>>>(&self, msg: S) {
        if self.0.module().contains(self.1) {
            self.log(Level::Error, msg)
        }
    }

    pub fn warn<S: Into<Vec<u8>>>(&self, msg: S) {
        if self.0.module().contains(self.1) {
            self.log(Level::Warn, msg)
        }
    }

    pub fn notice<S: Into<Vec<u8>>>(&self, msg: S) {
        if self.0.module().contains(self.1) {
            self.log(Level::Notice, msg)
        }
    }

    pub fn debug<S: Into<Vec<u8>>>(&self, msg: S) {
        if self.0.module().contains(self.1) {
            self.log(Level::Debug, msg)
        }
    }

    pub fn log<S: Into<Vec<u8>>>(&self, level: Level, msg: S) {
        if self.0.module().contains(self.1) {
            unsafe { self.0.error_core(level, None, msg) }
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    StdErr = ffi::NGX_LOG_STDERR,
    Emerg = ffi::NGX_LOG_EMERG,
    Alert = ffi::NGX_LOG_ALERT,
    Critical = ffi::NGX_LOG_CRIT,
    Error = ffi::NGX_LOG_ERR,
    Warn = ffi::NGX_LOG_WARN,
    Notice = ffi::NGX_LOG_NOTICE,
    Info = ffi::NGX_LOG_INFO,
    Debug = ffi::NGX_LOG_DEBUG,
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Module: u32 {
        const CORE = ffi::NGX_LOG_DEBUG_CORE;
        const ALLOC = ffi::NGX_LOG_DEBUG_ALLOC;
        const MUTEX = ffi::NGX_LOG_DEBUG_MUTEX;
        const EVENT = ffi::NGX_LOG_DEBUG_EVENT;
        const HTTP = ffi::NGX_LOG_DEBUG_HTTP;
        const MAIL = ffi::NGX_LOG_DEBUG_MAIL;
        const STREAM = ffi::NGX_LOG_DEBUG_STREAM;
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;
    use std::os::fd::AsRawFd;

    use super::*;

    #[test]
    fn log_stderr() {
        let log = Log::stderr();

        assert_eq!(log.level(), Level::Notice);
        assert_eq!(log.module(), Module::empty());
        assert_ne!(log.file().as_raw_fd(), 0);

        log.with_module(Module::CORE);
        log.core().notice("some test log");
    }

    #[test]
    fn log_tmp_file() {
        let tmp_dir = temp_dir();
        let log = Log::init(Some(&tmp_dir), Some("error.log")).unwrap();

        assert_eq!(log.level(), Level::Notice);
        assert_ne!(log.file().as_raw_fd(), 0);

        log.core().debug("test");
    }
}
