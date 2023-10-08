use foreign_types::foreign_type;

use crate::{ffi, never_drop};

foreign_type! {
    pub unsafe type LocConf: Send {
        type CType = ffi::ngx_http_core_loc_conf_t;

        fn drop = never_drop::<ffi::ngx_http_core_loc_conf_t>;
    }
}
