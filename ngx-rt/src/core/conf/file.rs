use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{core::BufRef, ffi, never_drop, AsRaw};

use super::ConfRef;

foreign_type! {
    pub unsafe type ConfFile: Send {
        type CType = ffi::ngx_conf_file_t;

        fn drop = never_drop::<ffi::ngx_conf_file_t>;
    }
}

impl ConfFileRef {
    pub fn buffer(&self) -> &BufRef {
        unsafe { BufRef::from_ptr(self.as_raw().buffer) }
    }

    pub fn dump(&self) -> &BufRef {
        unsafe { BufRef::from_ptr(self.as_raw().dump) }
    }

    pub fn line(&self) -> usize {
        unsafe { self.as_raw().line }
    }
}

impl ConfRef {
    pub fn conf_file(&self) -> &ConfFileRef {
        unsafe { ConfFileRef::from_ptr(self.as_raw().conf_file) }
    }
}
