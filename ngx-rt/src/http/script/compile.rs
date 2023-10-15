use foreign_types::foreign_type;

use crate::{ffi, never_drop};

foreign_type! {
    pub unsafe type Compile: Send {
        type CType = ffi::ngx_http_script_compile_t;

        fn drop = never_drop::<ffi::ngx_http_script_compile_t>;
    }
}
