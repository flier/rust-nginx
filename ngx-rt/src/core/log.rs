use std::{ffi::CString, path::Path, ptr};

use bitflags::bitflags;
use foreign_types::{foreign_type, ForeignTypeRef};
use num_enum::FromPrimitive;

use crate::{ffi, never_drop, AsRawMut, AsRawRef, Error, FromRawMut, Result};

use super::conf::OpenFileRef;

foreign_type! {
    pub unsafe type Log: Send {
        type CType = ffi::ngx_log_t;

        fn drop = never_drop::<ffi::ngx_log_t>;
    }
}

impl Log {
    pub fn stderr() -> &'static mut LogRef {
        #[cfg(feature = "static-link")]
        crate::core::time::init();

        unsafe {
            LogRef::from_ptr_mut(ffi::ngx_log_init(ptr::null_mut(), b"\0".as_ptr() as *mut _))
        }
    }

    pub fn init(
        prefix: Option<&'_ Path>,
        error_log: Option<&'_ str>,
    ) -> Result<&'static mut LogRef> {
        #[cfg(feature = "static-link")]
        crate::core::time::init();

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

    property! {
        file: &OpenFileRef;
    }

    pub fn level(&self) -> Level {
        Level::from(unsafe { self.as_raw().log_level as u32 & Self::LOG_LEVEL_MASK })
    }

    pub fn module(&self) -> Module {
        Module::from_bits_truncate(unsafe {
            self.as_raw().log_level as u32 & Self::LOG_MODULE_MASK
        })
    }

    pub fn with_module(&mut self, module: Module) {
        unsafe { self.as_raw_mut().log_level |= module.bits() as usize }
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

    pub fn error_core<S: Into<Vec<u8>>>(&self, level: Level, err: Option<i32>, msg: S) {
        unsafe {
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
}

impl Logger for LogRef {
    fn core<S: Into<Vec<u8>>>(&self, level: Level, msg: S) {
        self.error_core(level, None, msg)
    }
}

impl<T> Logger for T
where
    T: AsRef<LogRef>,
{
    fn core<S: Into<Vec<u8>>>(&self, level: Level, msg: S) {
        self.as_ref().error_core(level, None, msg)
    }
}

pub trait Logger {
    fn core<S: Into<Vec<u8>>>(&self, level: Level, msg: S);

    fn stderr<S: Into<Vec<u8>>>(&self, msg: S) {
        self.core(Level::StdErr, msg)
    }

    fn emerg<S: Into<Vec<u8>>>(&self, msg: S) {
        self.core(Level::Emerg, msg)
    }

    fn alert<S: Into<Vec<u8>>>(&self, msg: S) {
        self.core(Level::Alert, msg)
    }

    fn critical<S: Into<Vec<u8>>>(&self, msg: S) {
        self.core(Level::Critical, msg)
    }

    fn error<S: Into<Vec<u8>>>(&self, msg: S) {
        self.core(Level::Error, msg)
    }

    fn warn<S: Into<Vec<u8>>>(&self, msg: S) {
        self.core(Level::Warn, msg)
    }

    fn notice<S: Into<Vec<u8>>>(&self, msg: S) {
        self.core(Level::Notice, msg)
    }

    fn debug<S: Into<Vec<u8>>>(&self, msg: S) {
        self.core(Level::Debug, msg)
    }
}

pub struct WithModule<'a>(&'a LogRef, Module);

impl<'a> Logger for WithModule<'a> {
    fn core<S: Into<Vec<u8>>>(&self, level: Level, msg: S) {
        if self.0.module().contains(self.1) {
            self.0.error_core(level, None, msg)
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, FromPrimitive)]
pub enum Level {
    StdErr = ffi::NGX_LOG_STDERR,
    Emerg = ffi::NGX_LOG_EMERG,
    Alert = ffi::NGX_LOG_ALERT,
    Critical = ffi::NGX_LOG_CRIT,
    Error = ffi::NGX_LOG_ERR,
    Warn = ffi::NGX_LOG_WARN,
    #[default]
    Notice = ffi::NGX_LOG_NOTICE,
    Info = ffi::NGX_LOG_INFO,
    Debug = ffi::NGX_LOG_DEBUG,
}

macro_rules! define_logger {
    ( $( $name:ident => $level:ident ,)* ) => {
        define_logger! { __impl =>
            ($d:tt) => {
                $(
                    #[macro_export]
                    macro_rules! $name {
                        ($d log:expr, $d( $d args:tt )*) => {
                            $d crate::core::Logger::core(
                                & $d log,
                                $crate::core::LogLevel::$level,
                                format!($d ($d args)*)
                            )
                        };
                    }
                )*
            }
        }
    };
    ( __impl => $($body:tt)* ) => {
        macro_rules! __with_dollar_sign { $($body)* }
        __with_dollar_sign!($);
    }
}

define_logger! {
    stderr => StdErr,
    emerg => Emerg,
    alert => Alert,
    critical => Critical,
    error => Error,
    warn => Warn,
    notice => Notice,
    info => Info,
    debug => Debug,
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
        notice!(log.core(), "some test log");
    }

    #[test]
    fn log_tmp_file() {
        let tmp_dir = temp_dir();
        let log = Log::init(Some(&tmp_dir), Some("error.log")).unwrap();

        assert_eq!(log.level(), Level::Notice);
        assert_ne!(log.file().as_raw_fd(), 0);

        debug!(log.core(), "test");
    }
}
