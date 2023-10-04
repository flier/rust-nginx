use std::{
    ffi::{c_char, c_void},
    ptr,
};

use foreign_types::ForeignTypeRef;

use crate::{
    rt::{
        core::{CycleRef, NGX_CONF_ERROR, NGX_CONF_OK},
        ffi,
    },
    Merge,
};

pub trait UnsafeModule {
    /// Create the configuration.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe extern "C" fn create_conf(cycle: *mut ffi::ngx_cycle_t) -> *mut c_void;

    /// Initialize the configuration.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe extern "C" fn init_conf(cycle: *mut ffi::ngx_cycle_t, conf: *mut c_void) -> *mut c_char;
}

impl<T: Module> UnsafeModule for T {
    unsafe extern "C" fn create_conf(cycle: *mut ffi::ngx_cycle_t) -> *mut c_void {
        <T as Module>::create_conf(CycleRef::from_ptr(cycle))
            .map_or_else(ptr::null_mut, |p| p as *mut _ as *mut _)
    }

    unsafe extern "C" fn init_conf(cycle: *mut ffi::ngx_cycle_t, conf: *mut c_void) -> *mut c_char {
        <T as Module>::init_conf(CycleRef::from_ptr(cycle), &mut *conf.cast())
            .map_or(NGX_CONF_ERROR, |_| NGX_CONF_OK)
    }
}

pub trait Module {
    type Error: From<<Self::Conf as Merge>::Error>;
    type Conf: Default + Merge;

    fn create_conf(cycle: &CycleRef) -> Option<&mut Self::Conf> {
        cycle.pool().allocate(Self::Conf::default())
    }

    fn init_conf(_cycle: &CycleRef, _conf: &mut Self::Conf) -> Result<(), Self::Error> {
        Ok(())
    }
}
