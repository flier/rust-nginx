use std::time::Duration;

use crate::{core::Str, ffi};

pub fn parse_size<S: AsRef<str>>(s: S) -> Option<u64> {
    let s = s.as_ref();
    let s = Str::from(s);
    let n = unsafe { ffi::ngx_parse_size(&s as *const _ as *mut _) };

    if n < 0 {
        None
    } else {
        Some(n as u64)
    }
}

pub fn parse_offset<S: AsRef<str>>(s: S) -> Option<u32> {
    let s = s.as_ref();
    let s = Str::from(s);
    let n = unsafe { ffi::ngx_parse_offset(&s as *const _ as *mut _) };

    if n < 0 {
        None
    } else {
        Some(n as u32)
    }
}

pub fn parse_time<S: AsRef<str>>(s: S) -> Option<Duration> {
    let s = s.as_ref();
    let s = Str::from(s);
    let n = unsafe { ffi::ngx_parse_time(&s as *const _ as *mut _, 0) };

    if n < 0 {
        None
    } else {
        Some(Duration::from_millis(n as u64))
    }
}
