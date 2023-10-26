use std::ffi::{c_char, CString};

use errno::{errno, Errno};
use thiserror::Error;

use crate::{core, ffi};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("out of memory")]
    OutOfMemory,

    #[error(transparent)]
    Errno(#[from] Errno),

    #[error("internal error, {0}")]
    InternalError(isize),

    #[error("config error, {0:?}")]
    ConfigError(CString),

    #[error(transparent)]
    NulError(#[from] std::ffi::NulError),
}

impl Error {
    pub fn errno() -> Self {
        Self::Errno(errno())
    }
}

impl From<isize> for Error {
    fn from(value: isize) -> Self {
        Self::InternalError(value)
    }
}

pub trait RawResult<O> {
    type Output;
    type Error;

    fn raw_result(self) -> O;
}

impl<T, E, O> RawResult<O> for std::result::Result<T, E>
where
    T: RawOk<O>,
    E: RawErr<O>,
{
    type Output = T;
    type Error = E;

    fn raw_result(self) -> O {
        match self {
            Ok(ok) => ok.raw_ok(),
            Err(err) => err.raw_err(),
        }
    }
}

pub trait RawOk<O> {
    fn raw_ok(self) -> O;
}

pub trait RawErr<O> {
    fn raw_err(self) -> O;
}

impl RawOk<ffi::ngx_int_t> for () {
    fn raw_ok(self) -> ffi::ngx_int_t {
        ffi::NGX_OK as ffi::ngx_int_t
    }
}

impl RawErr<ffi::ngx_int_t> for () {
    fn raw_err(self) -> ffi::ngx_int_t {
        ffi::NGX_ERROR as ffi::ngx_int_t
    }
}

impl RawOk<*mut c_char> for () {
    fn raw_ok(self) -> *mut c_char {
        core::NGX_CONF_OK as *mut c_char
    }
}

impl RawErr<*mut c_char> for () {
    fn raw_err(self) -> *mut c_char {
        core::NGX_CONF_ERROR as *mut c_char
    }
}

impl RawOk<ffi::ngx_int_t> for ::http::StatusCode {
    fn raw_ok(self) -> ffi::ngx_int_t {
        self.as_u16() as ffi::ngx_int_t
    }
}
