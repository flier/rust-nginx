use foreign_types::foreign_type;

use crate::{ffi, flag, never_drop, property, AsRawRef};

use super::{BufRef, LogRef, PoolRef};

foreign_type! {
    pub unsafe type Conn: Send {
        type CType = ffi::ngx_connection_t;

        fn drop = never_drop::<ffi::ngx_connection_t>;
    }
}

impl ConnRef {
    property!(log: &LogRef);
    property!(pool: &PoolRef);
    property!(buffer: &BufRef);

    property!(requests: usize);

    pub fn log_error(&self) -> LogError {
        match unsafe { self.as_raw().log_error() } {
            0 => LogError::Alert,
            1 => LogError::Error,
            2 => LogError::Info,
            3 => LogError::IgnoreConnReset,
            4 => LogError::IgnoreInvalid,
            5 => LogError::IgnoreMsgSize,
            _ => unreachable!(),
        }
    }

    flag!(timedout());
    flag!(error());
    flag!(destroyed());
    flag!(pipeline());

    flag!(idle());
    flag!(reusable());
    flag!(close());
    flag!(shared());

    flag!(sendfile());
    flag!(sndlowat());

    pub fn tcp_nodelay(&self) -> Option<TcpNoDelay> {
        match unsafe { self.as_raw().tcp_nodelay() } {
            1 => Some(TcpNoDelay::Set),
            2 => Some(TcpNoDelay::Disabled),
            _ => None,
        }
    }

    pub fn tcp_nopush(&self) -> Option<TcpNoPush> {
        match unsafe { self.as_raw().tcp_nopush() } {
            1 => Some(TcpNoPush::Set),
            2 => Some(TcpNoPush::Disabled),
            _ => None,
        }
    }

    flag!(need_last_buf());
    flag!(need_flush_buf());
}

pub enum LogError {
    Alert = 0,
    Error,
    Info,
    IgnoreConnReset,
    IgnoreInvalid,
    IgnoreMsgSize,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TcpNoDelay {
    Set = 1,
    Disabled,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TcpNoPush {
    Set = 1,
    Disabled,
}
