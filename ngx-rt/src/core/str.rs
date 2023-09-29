use std::borrow::Cow;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::slice;
use std::str::{self, Utf8Error};

use crate::ffi::{ngx_str_t, u_char};

#[repr(transparent)]
#[derive(Debug)]
pub struct Str([u_char]);

impl Str {
    /// Create an [`Str`] from an [`ngx_str_t`].
    ///
    /// [`ngx_str_t`]: https://nginx.org/en/docs/dev/development_guide.html#string_overview
    ///
    /// # Safety
    ///
    /// The caller has provided a valid `ngx_str_t` with a `data` pointer that points
    /// to range of bytes of at least `len` bytes, whose content remains valid and doesn't
    /// change for the lifetime of the returned `Str`.
    pub unsafe fn from_raw<'a>(str: ngx_str_t) -> Option<&'a Self> {
        NonNull::new(str.data).map(|p| slice::from_raw_parts(p.as_ptr(), str.len).into())
    }

    /// Create an [`Str`] from an pointer of [`ngx_str_t`].
    ///
    /// [`ngx_str_t`]: https://nginx.org/en/docs/dev/development_guide.html#string_overview
    ///
    /// # Safety
    ///
    /// The caller has provided a valid `ngx_str_t` with a `data` pointer that points
    /// to range of bytes of at least `len` bytes, whose content remains valid and doesn't
    /// change for the lifetime of the returned `Str`.
    pub unsafe fn from_ptr<'a>(str: *mut ngx_str_t) -> Option<&'a Self> {
        NonNull::new(str).and_then(|p| Self::from_raw(*p.as_ref()))
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

impl fmt::Display for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_lossy())
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
        unsafe { *bytes.as_ptr().cast::<Self>() }
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
