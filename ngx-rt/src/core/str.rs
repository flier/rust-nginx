use std::borrow::Cow;
use std::ops::{Deref, DerefMut};
use std::slice;
use std::str::{self, Utf8Error};

use crate::ffi::{ngx_str_t, u_char};

#[repr(transparent)]
pub struct Str([u_char]);

impl Str {
    pub unsafe fn from_raw<'a>(str: ngx_str_t) -> &'a Self {
        slice::from_raw_parts(str.data, str.len).into()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }

    pub fn as_str(&self) -> Result<Option<&str>, Utf8Error> {
        if self.is_empty() {
            Ok(None)
        } else {
            str::from_utf8(self.as_bytes()).map(Some)
        }
    }

    pub fn to_str(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(self.as_bytes())
    }

    pub fn to_string_lossy(&self) -> Cow<str> {
        String::from_utf8_lossy(self.as_bytes())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<&Str> for ngx_str_t {
    fn from(s: &Str) -> ngx_str_t {
        ngx_str_t {
            len: s.len(),
            data: s.as_ptr() as *mut u_char,
        }
    }
}

impl From<&[u8]> for &Str {
    fn from(bytes: &[u8]) -> Self {
        unsafe { &*bytes.as_ptr().cast::<Self>() }
    }
}

impl From<&str> for &Str {
    fn from(s: &str) -> Self {
        s.as_bytes().into()
    }
}

impl AsRef<[u8]> for Str {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsMut<[u8]> for Str {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_bytes_mut()
    }
}

impl Deref for Str {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl DerefMut for Str {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_bytes_mut()
    }
}

impl Default for &Str {
    fn default() -> Self {
        unsafe { Str::from_raw(crate::ngx_str!()) }
    }
}

#[macro_export]
macro_rules! ngx_str {
    () => {{
        $crate::ffi::ngx_str_t {
            len: 0,
            data: ::std::ptr::null_mut(),
        }
    }};

    ($s:literal) => {{
        $crate::ffi::ngx_str_t {
            len: $s.len() as _,
            data: concat!($s, "\0").as_ptr() as *mut $crate::ffi::u_char,
        }
    }};
}
