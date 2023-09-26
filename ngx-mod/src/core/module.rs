use std::{
    ffi::{c_char, c_void},
    ptr::{self, NonNull},
};

use foreign_types::ForeignTypeRef;

use ngx_rt::core::{CycleRef, NGX_CONF_ERROR, NGX_CONF_OK};

use crate::{ffi, Merge};

pub trait UnsafeModule {
    unsafe extern "C" fn create_conf(cycle: *mut ffi::ngx_cycle_t) -> *mut c_void;

    unsafe extern "C" fn init_conf(cycle: *mut ffi::ngx_cycle_t, conf: *mut c_void) -> *mut c_char;
}

impl<T: Module> UnsafeModule for T {
    unsafe extern "C" fn create_conf(cycle: *mut ffi::ngx_cycle_t) -> *mut c_void {
        <T as Module>::create_conf(CycleRef::from_ptr(cycle))
            .map_or_else(ptr::null_mut, |p| p.as_ptr().cast())
    }

    unsafe extern "C" fn init_conf(cycle: *mut ffi::ngx_cycle_t, conf: *mut c_void) -> *mut c_char {
        <T as Module>::init_conf(CycleRef::from_ptr(cycle), &mut *conf.cast())
            .map_or(NGX_CONF_ERROR, |_| NGX_CONF_OK)
    }
}

pub trait Module {
    type Error: From<<Self::Conf as Merge>::Error>;
    type Conf: Default + Merge;

    fn create_conf(cycle: &CycleRef) -> Option<NonNull<Self::Conf>> {
        cycle.pool().allocate(Self::Conf::default())
    }

    fn init_conf(_cycle: &CycleRef, _conf: &mut Self::Conf) -> Result<(), Self::Error> {
        Ok(())
    }
}
