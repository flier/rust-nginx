use std::sync::Once;

use crate::ffi;

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
