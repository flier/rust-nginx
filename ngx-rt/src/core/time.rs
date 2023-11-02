use std::ptr::read_volatile;
use std::sync::Once;
use std::time::Duration;

use derive_more::{Deref, DerefMut, Display, From, Into};

use crate::{
    core::{rbtree, Unset},
    ffi,
};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Deref, DerefMut, Display, From, Into)]
#[display(fmt = "{}", _0)]
pub struct MSec(rbtree::Key);

impl Unset for MSec {
    const UNSET: Self = MSec(usize::MAX);

    fn is_unset(&self) -> bool {
        *self == Self::UNSET
    }
}

impl From<MSec> for Duration {
    fn from(sec: MSec) -> Self {
        Duration::from_millis(sec.0 as u64)
    }
}

impl From<Duration> for MSec {
    fn from(d: Duration) -> Self {
        Self(d.as_millis() as usize)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Deref, DerefMut, Display, From, Into)]
#[display(fmt = "{}", _0)]
pub struct Sec(ffi::time_t);

impl Unset for Sec {
    const UNSET: Self = Sec(u64::MAX as ffi::time_t);

    fn is_unset(&self) -> bool {
        *self == Self::UNSET
    }
}

impl From<Sec> for Duration {
    fn from(sec: Sec) -> Self {
        Duration::from_secs(sec.0 as u64)
    }
}

impl From<Duration> for Sec {
    fn from(d: Duration) -> Self {
        Self(d.as_secs() as ffi::time_t)
    }
}

static INIT: Once = Once::new();

pub fn init() {
    INIT.call_once(|| unsafe {
        ffi::ngx_time_init();
    })
}

pub fn update() {
    unsafe {
        ffi::ngx_time_update();
    }
}

pub fn current() -> Duration {
    Duration::from_millis(unsafe { read_volatile(&ffi::ngx_current_msec as *const _) as _ })
}
