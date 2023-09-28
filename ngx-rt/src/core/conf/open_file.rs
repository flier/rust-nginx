use std::os::fd::{AsRawFd, RawFd};

use foreign_types::foreign_type;

use crate::{core::Str, ffi, never_drop, AsRawRef};

foreign_type! {
    pub unsafe type OpenFile: Send {
        type CType = ffi::ngx_open_file_t;

        fn drop = never_drop::<ffi::ngx_open_file_t>;
    }
}

impl AsRawFd for OpenFileRef {
    fn as_raw_fd(&self) -> RawFd {
        unsafe { self.as_raw_ref().fd }
    }
}

impl OpenFileRef {
    pub fn name(&self) -> Option<&Str> {
        unsafe { Str::from_raw(self.as_raw_ref().name) }
    }
}
