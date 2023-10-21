use std::os::fd::{AsRawFd, RawFd};

use foreign_types::foreign_type;

use crate::{ffi, never_drop, AsRawRef};

foreign_type! {
    pub unsafe type OpenFile: Send {
        type CType = ffi::ngx_open_file_t;

        fn drop = never_drop::<ffi::ngx_open_file_t>;
    }
}

impl AsRawFd for OpenFileRef {
    fn as_raw_fd(&self) -> RawFd {
        unsafe { self.as_raw().fd }
    }
}

impl OpenFileRef {
    str! {
        &name
    }
}
