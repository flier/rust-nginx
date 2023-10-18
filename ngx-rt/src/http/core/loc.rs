use std::ptr::NonNull;

use foreign_types::foreign_type;

use crate::{ffi, flag, http::UnsafeLocConf, never_drop, str, AsRawRef};

foreign_type! {
    pub unsafe type LocConf: Send {
        type CType = ffi::ngx_http_core_loc_conf_t;

        fn drop = never_drop::<ffi::ngx_http_core_loc_conf_t>;
    }
}

impl LocConfRef {
    str! {
        &name;
        &escaped_name;
    }

    flag! {
        /// "if () {}" block or limit_except
        noname;
        lmt_excpt;
        named;

        exact_match;
        noregex;

        auto_redirect;
    }
}

impl UnsafeLocConf for LocConfRef {
    unsafe fn unchecked_loc_conf<T>(&self, idx: usize) -> Option<NonNull<T>> {
        NonNull::new(self.as_raw().loc_conf.add(idx).read().cast())
    }
}
