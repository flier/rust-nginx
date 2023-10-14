use foreign_types::foreign_type;
use ngx_rt_derive::native_callback;

use crate::{callback, core::LogRef, ffi, flag, never_drop, property};

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
        timer_set;

        delayed;

        deferred_accept;

        /// the pending eof reported by kqueue, epoll or in aio chain operation
        pending_eof;

        posted;

        closed;

        /// to test on worker exit
        channel;
        resolver;

        cancelable;
    }

    callback! {
        handler: HandlerFn;
    }

    property! {
        index: usize;
        log: &LogRef;
    }
}

#[native_callback]
pub type HandlerFn = fn(evt: &EventRef);
