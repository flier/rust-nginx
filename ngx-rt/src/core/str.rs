use std::borrow::Cow;
use std::ffi::c_uchar;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::ptr::{self, null_mut, NonNull};
use std::slice;
use std::str::{self, Utf8Error};

use derive_more::{AsMut, AsRef, From, Into};
use foreign_types::ForeignTypeRef;

use crate::{core::PoolRef, ffi};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, AsRef, AsMut, From, Into)]
pub struct Str(ffi::ngx_str_t);

impl Str {
    pub const NULL: Self = Self::null();

    pub const fn null() -> Self {
        Self(ffi::ngx_str_t {
            len: 0,
            data: ::std::ptr::null_mut(),
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
        Str(ffi::ngx_str_t {
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
        self.0.data.is_null() || self.0.len == 0
    }

    pub fn len(&self) -> usize {
        self.0.len
    }
}

impl Default for Str {
    fn default() -> Self {
        Str(ffi::ngx_str_t {
            len: 0,
            data: null_mut(),
        })
    }
}

impl Hash for Str {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state)
    }
}

impl fmt::Display for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_lossy())
    }
}

impl From<&[u8]> for Str {
    fn from(bytes: &[u8]) -> Self {
        Str(ffi::ngx_str_t {
            len: bytes.len(),
            data: bytes.as_ptr().cast::<c_uchar>() as *mut _,
        })
    }
}

impl From<&Str> for ffi::ngx_str_t {
    fn from(s: &Str) -> Self {
        ffi::ngx_str_t {
            len: s.len(),
            data: s.as_ptr().cast::<c_uchar>() as *mut _,
        }
    }
}

impl From<&str> for Str {
    fn from(s: &str) -> Self {
        Str(ffi::ngx_str_t {
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

impl PartialEq<ffi::ngx_str_t> for Str {
    fn eq(&self, other: &ffi::ngx_str_t) -> bool {
        unsafe { self.as_bytes() == slice::from_raw_parts(other.data, other.len) }
    }
}

impl Eq for Str {}
impl PartialEq<Str> for Str {
    fn eq(&self, other: &Str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<&str> for Str {
    fn eq(&self, other: &&str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

unsafe impl ForeignTypeRef for Str {
    type CType = ffi::ngx_str_t;
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

                Str(ffi::ngx_str_t {
                    len: s.len(),
                    data: p.as_ptr(),
                })
            })
        }
    }
}
