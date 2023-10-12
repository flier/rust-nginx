use core::fmt;

use http::StatusCode;

use crate::{core::strerror, ffi, RawErr, RawOk};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Code(i32);

impl Code {
    /// Operation succeeded.
    pub const OK: Code = Code(ffi::NGX_OK as i32);
    /// Operation failed.
    pub const ERROR: Code = Code(ffi::NGX_ERROR);
    /// Operation incomplete; call the function again.
    pub const AGAIN: Code = Code(ffi::NGX_AGAIN);
    /// Resource is not available.
    pub const BUSY: Code = Code(ffi::NGX_BUSY);
    /// Operation complete or continued elsewhere.
    ///
    /// Also used as an alternative success code.
    pub const DONE: Code = Code(ffi::NGX_DONE);
    /// Operation rejected
    pub const DECLINED: Code = Code(ffi::NGX_DECLINED);
    /// Function was aborted.
    ///
    /// Also used as an alternative error code.
    pub const ABORT: Code = Code(ffi::NGX_ABORT);

    pub fn is_ok(&self) -> bool {
        self.0 == ffi::NGX_OK as i32
    }

    pub fn as_status_code(&self) -> Option<StatusCode> {
        if self.0 > 0 {
            StatusCode::from_u16(self.0 as u16).ok()
        } else {
            None
        }
    }

    pub fn err(&self) -> Option<i32> {
        if self.is_ok() {
            None
        } else {
            Some(self.0)
        }
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(code) = self.as_status_code() {
            write!(f, "{}", code)
        } else {
            write!(f, "{}", strerror(self.0).to_string_lossy())
        }
    }
}

impl From<i32> for Code {
    fn from(n: i32) -> Self {
        Self(n)
    }
}

impl From<isize> for Code {
    fn from(n: isize) -> Self {
        Self(n as i32)
    }
}

impl From<StatusCode> for Code {
    fn from(code: StatusCode) -> Self {
        Self(code.as_u16() as i32)
    }
}

impl TryFrom<Code> for StatusCode {
    type Error = Code;

    fn try_from(code: Code) -> Result<Self, Self::Error> {
        if code.0 > 0 {
            StatusCode::from_u16(code.0 as u16).map_err(|_| code)
        } else {
            Err(code)
        }
    }
}

impl From<Code> for i32 {
    fn from(code: Code) -> Self {
        code.0
    }
}

impl From<Code> for ffi::ngx_int_t {
    fn from(code: Code) -> Self {
        code.0 as ffi::ngx_int_t
    }
}

impl RawErr<ffi::ngx_int_t> for Code {
    fn raw_err(self) -> ffi::ngx_int_t {
        self.0 as ffi::ngx_int_t
    }
}

impl RawOk<ffi::ngx_int_t> for Code {
    fn raw_ok(self) -> ffi::ngx_int_t {
        self.0 as ffi::ngx_int_t
    }
}
