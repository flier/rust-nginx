use std::os::fd::AsRawFd;
use std::ptr::null_mut;
use std::sync::Once;
use std::time::Duration;

use foreign_types::ForeignTypeRef;

use crate::{
    core::{rbtree, time},
    ffi, AsRawMut,
};

use super::EventRef;

const LAZY_DELAY: Duration = Duration::from_millis(300);

static INIT: Once = Once::new();

pub fn init() {
    INIT.call_once(|| unsafe {
        ffi::ngx_event_timer_init(null_mut());
    })
}

pub fn find_timer() -> Option<Duration> {
    #[cfg(feature = "static-link")]
    init();

    const NGX_TIMER_INFINITE: usize = usize::MAX;

    unsafe {
        let t = ffi::ngx_event_find_timer();

        if t == NGX_TIMER_INFINITE {
            None
        } else {
            Some(Duration::from_millis(t as u64))
        }
    }
}

pub fn expire_timers() {
    #[cfg(feature = "static-link")]
    init();

    unsafe { ffi::ngx_event_expire_timers() }
}

pub fn no_timers_left() -> bool {
    #[cfg(feature = "static-link")]
    init();

    unsafe { ffi::ngx_event_no_timers_left() == ffi::NGX_OK as isize }
}

pub fn rbtree() -> &'static mut rbtree::TreeRef {
    #[cfg(feature = "static-link")]
    init();

    unsafe { rbtree::TreeRef::from_ptr_mut(&mut ffi::ngx_event_timer_rbtree as *mut _) }
}

impl EventRef {
    pub fn del_timer(&mut self) {
        debug!(
            self.log().event(),
            "event timer del: {}: {:?}",
            self.as_raw_fd(),
            self.timer().key()
        );

        rbtree().delete_node(self.timer_mut());

        unsafe {
            if cfg!(debug_assertions) {
                let t = &mut self.as_raw_mut().timer;

                t.left = null_mut();
                t.right = null_mut();
                t.parent = null_mut();
            }

            self.as_raw_mut().set_timer_set(0);
        }
    }

    pub fn add_timer(&mut self, d: Duration) {
        let key = time::current() + d;

        if self.timer_set() {
            let old_key = Duration::from_millis(self.timer().key() as u64);

            /*
             * Use a previous timer value if difference between it and a new
             * value is less than NGX_TIMER_LAZY_DELAY milliseconds: this allows
             * to minimize the rbtree operations for fast connections.
             */

            let diff = if key > old_key {
                key - old_key
            } else {
                old_key - key
            };

            if diff < LAZY_DELAY {
                debug!(
                    self.log().event(),
                    "event timer: {}, old: {:?}, new: {:?}",
                    self.as_raw_fd(),
                    old_key,
                    key
                );

                return;
            }

            self.del_timer();
        }

        self.timer_mut().set_key(key.as_millis() as usize);

        debug!(
            self.log().event(),
            "event timer add: {}: {:?}:{:?}",
            self.as_raw_fd(),
            d,
            key
        );

        rbtree().insert_node(self.timer_mut());

        self.set_timer_set(true);
    }
}

#[cfg(test)]
mod tests {
    use std::mem::zeroed;

    use crate::core::Log;

    use super::*;

    #[test]
    fn timer() {
        // the timer rbtree should be empty

        assert!(no_timers_left());
        assert!(find_timer().is_none());

        // create a empty event

        let evt = unsafe {
            let mut evt: ffi::ngx_event_t = zeroed();

            evt.log = Log::stderr().as_ptr();

            EventRef::from_ptr_mut(&mut evt as *mut _)
        };

        let d = Duration::from_millis(500);

        // add the timer to the rbtree

        evt.add_timer(d);

        // the timer should be in the rbtree

        assert!(evt.timer_set());
        assert!(!no_timers_left());
        assert_eq!(find_timer().unwrap(), d);

        // add the same event with different timeout

        evt.add_timer(d * 2);

        assert_eq!(find_timer().unwrap(), d * 2);

        // change the event to cancelable, then the event should not be counted in timers

        evt.set_cancelable(true);
        assert!(no_timers_left());

        // restore the event

        evt.set_cancelable(false);
        assert!(!no_timers_left());

        // delete the timer

        evt.del_timer();

        // the timer should not be in the rbtree

        assert!(no_timers_left());
        assert!(find_timer().is_none());
    }
}
