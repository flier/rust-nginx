use std::ffi::CString;

use crate::ffi;

pub use errno::{errno, Errno};

pub fn strerror(err: ffi::ngx_err_t) -> CString {
    #[cfg(feature = "static-link")]
    {
        use std::sync::Once;

        static INIT: Once = Once::new();

        INIT.call_once(|| unsafe {
            ffi::ngx_strerror_init();
        })
    }

    let mut buf = Vec::with_capacity(ffi::NGX_MAX_ERROR_STR as usize);

    unsafe {
        ffi::ngx_strerror(err, buf.as_mut_ptr(), buf.len());

        CString::from_vec_with_nul_unchecked(buf)
    }
}
