use std::os::fd::{AsRawFd, RawFd};
use std::ptr::NonNull;

use foreign_types::foreign_type;
use ngx_rt_derive::native_callback;

use crate::{
    callback,
    core::{rbtree, ConnRef, LogRef},
    ffi, flag, never_drop, property, AsRawRef,
};

foreign_type! {
    pub unsafe type Event: Send {
        type CType = ffi::ngx_event_t;

        fn drop = never_drop::<ffi::ngx_event_t>;
    }
}

impl EventRef {
    flag! {
        write;
        accept;

        /// used to detect the stale events in kqueue and epoll
        instance;

        /// the event was passed or would be passed to a kernel; in aio mode - operation was posted.
        active;

        disabled;

        /// the ready event; in aio mode 0 means that no operation can be posted
        ready;

        oneshot;

        /// aio operation is complete
        complete;

        eof;
        error;

        timedout;
        timer_set { get; set; };

        delayed;

        deferred_accept;

        /// the pending eof reported by kqueue, epoll or in aio chain operation
        pending_eof;

        posted;

        closed;

        /// to test on worker exit
        channel;
        resolver;

        cancelable { get; set; };
    }

    callback! {
        handler: HandlerFn;
    }

    property! {
        index: usize;
        log: &LogRef;
        &mut timer: &mut rbtree::NodeRef;
    }

    pub fn data<T>(&self) -> Option<&T> {
        unsafe { NonNull::new(self.as_raw().data).map(|p| p.cast::<T>().as_ref()) }
    }
}

#[native_callback]
pub type HandlerFn = fn(evt: &EventRef);

impl AsRef<LogRef> for EventRef {
    fn as_ref(&self) -> &LogRef {
        self.log()
    }
}

impl AsRawFd for EventRef {
    fn as_raw_fd(&self) -> RawFd {
        self.data::<ConnRef>().map_or(-1, |c| c.as_raw_fd())
    }
}
