use std::ptr::read_volatile;
use std::sync::Once;
use std::time::Duration;

use crate::ffi;

use super::rbtree;

pub type MSec = rbtree::Key;

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
