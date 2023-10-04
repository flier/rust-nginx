use std::ffi::{c_char, c_void};

use foreign_types::ForeignTypeRef;

use crate::rt::{
    core::{CmdRef, ConfRef, NGX_CONF_ERROR, NGX_CONF_OK},
    ffi,
};

pub trait UnsafeSetter {
    /// Set the configuration.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe extern "C" fn set(
        _cf: *mut ffi::ngx_conf_t,
        cmd: *mut ffi::ngx_command_t,
        conf: *mut c_void,
    ) -> *mut c_char;
}

impl<T: Setter> UnsafeSetter for T {
    unsafe extern "C" fn set(
        cf: *mut ffi::ngx_conf_t,
        cmd: *mut ffi::ngx_command_t,
        conf: *mut c_void,
    ) -> *mut c_char {
        T::set(
            ConfRef::from_ptr(cf),
            CmdRef::from_ptr(cmd),
            &mut *conf.cast(),
        )
        .map_or(NGX_CONF_ERROR, |_| NGX_CONF_OK)
    }
}

pub trait Setter {
    type Error;
    type Conf;

    fn set(_cf: &ConfRef, _cmd: &CmdRef, _conf: &mut Self::Conf) -> Result<(), Self::Error> {
        Ok(())
    }
}
