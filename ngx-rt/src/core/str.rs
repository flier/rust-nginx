use std::borrow::Cow;
use std::ffi::c_uchar;
use std::ops::{Deref, DerefMut};
use std::ptr::{null_mut, NonNull};
use std::slice;
use std::str::{self, Utf8Error};
use std::{fmt, ptr};

use crate::ffi::ngx_str_t;

use super::PoolRef;

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct Str(ngx_str_t);

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
    pub unsafe fn from_raw(str: ngx_str_t) -> Option<Self> {
        if str.data.is_null() {
            None
        } else {
            Some(Str(str))
        }
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
    pub unsafe fn from_ptr(str: *mut ngx_str_t) -> Option<Self> {
        NonNull::new(str).and_then(|p| {
            let s = p.as_ref();

            if s.data.is_null() {
                None
            } else {
                Some(Str(*s))
            }
        })
    }

    /// Create an [`Str`] from a memory pointer and size.
    ///
    /// # Safety
    ///
    /// The caller has provided a valid `data` pointer that points
    /// to range of bytes of at least `len` bytes, whose content remains valid and doesn't
    /// change for the lifetime of the returned `Str`.
    pub fn unchecked_new(data: NonNull<c_uchar>, len: usize) -> Self {
        Str(ngx_str_t {
            len,
            data: data.as_ptr(),
        })
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.0.data, self.0.len) }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.0.data, self.0.len) }
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

    pub fn is_null(&self) -> bool {
        self.0.data.is_null()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len == 0
    }

    pub fn len(&self) -> usize {
        self.0.len
    }
}

impl Default for Str {
    fn default() -> Self {
        Str(ngx_str_t {
            len: 0,
            data: null_mut(),
        })
    }
}

impl fmt::Display for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_lossy())
    }
}

impl From<ngx_str_t> for Str {
    fn from(str: ngx_str_t) -> Self {
        Self(str)
    }
}

impl From<Str> for ngx_str_t {
    fn from(str: Str) -> Self {
        str.0
    }
}

impl From<&[u8]> for Str {
    fn from(bytes: &[u8]) -> Self {
        Str(ngx_str_t {
            len: bytes.len(),
            data: bytes.as_ptr().cast::<c_uchar>() as *mut _,
        })
    }
}

impl From<&str> for Str {
    fn from(s: &str) -> Self {
        Str(ngx_str_t {
            len: s.len(),
            data: s.as_ptr().cast::<c_uchar>() as *mut _,
        })
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

impl PoolRef {
    pub fn strdup<S: AsRef<str>>(&self, s: S) -> Option<Str> {
        let s = s.as_ref();

        unsafe {
            NonNull::new(self.pnalloc(s.len())).map(|p| {
                let p = p.cast();

                ptr::copy_nonoverlapping(s.as_ptr(), p.as_ptr(), s.len());

                Str(ngx_str_t {
                    len: s.len(),
                    data: p.as_ptr(),
                })
            })
        }
    }
}
