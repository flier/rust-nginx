use foreign_types::foreign_type;

use crate::{core::BufRef, ffi, never_drop, property};

use super::ConfRef;

foreign_type! {
    pub unsafe type ConfFile: Send {
        type CType = ffi::ngx_conf_file_t;

        fn drop = never_drop::<ffi::ngx_conf_file_t>;
    }
}

impl ConfFileRef {
    property! {
        buffer: &BufRef;
        dump: &BufRef;
        line: usize;
    }
}

impl ConfRef {
    property! {
        conf_file: &ConfFileRef
    }
}
