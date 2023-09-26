use std::{
    ffi::{c_char, c_void},
    ptr::{self, NonNull},
};

use foreign_types::ForeignTypeRef;

use crate::{
    ffi,
    rt::core::{Code, ConfRef},
    Merge,
};

pub const NGX_CONF_OK: *mut c_char = ptr::null_mut();
pub const NGX_CONF_ERROR: *mut c_char = usize::MAX as *mut c_char;

pub trait UnsafeModule {
    unsafe extern "C" fn preconfiguration(cf: *mut ffi::ngx_conf_t) -> ffi::ngx_int_t;

    unsafe extern "C" fn postconfiguration(cf: *mut ffi::ngx_conf_t) -> ffi::ngx_int_t;

    unsafe extern "C" fn create_main_conf(cf: *mut ffi::ngx_conf_t) -> *mut c_void;

    unsafe extern "C" fn init_main_conf(cf: *mut ffi::ngx_conf_t, conf: *mut c_void)
        -> *mut c_char;

    unsafe extern "C" fn create_srv_conf(cf: *mut ffi::ngx_conf_t) -> *mut c_void;

    unsafe extern "C" fn merge_srv_conf(
        cf: *mut ffi::ngx_conf_t,
        prev: *mut c_void,
        conf: *mut c_void,
    ) -> *mut c_char;

    unsafe extern "C" fn create_loc_conf(cf: *mut ffi::ngx_conf_t) -> *mut c_void;

    unsafe extern "C" fn merge_loc_conf(
        cf: *mut ffi::ngx_conf_t,
        prev: *mut c_void,
        conf: *mut c_void,
    ) -> *mut c_char;
}

impl<T: Module> UnsafeModule for T {
    unsafe extern "C" fn preconfiguration(cf: *mut ffi::ngx_conf_t) -> ffi::ngx_int_t {
        <T as Module>::preconfiguration(ConfRef::from_ptr(cf))
            .map(|_| Code::Ok)
            .unwrap_or_else(|code| code) as ffi::ngx_int_t
    }

    unsafe extern "C" fn postconfiguration(cf: *mut ffi::ngx_conf_t) -> ffi::ngx_int_t {
        <T as Module>::postconfiguration(ConfRef::from_ptr(cf))
            .map(|_| Code::Ok)
            .unwrap_or_else(|code| code) as ffi::ngx_int_t
    }

    unsafe extern "C" fn create_main_conf(cf: *mut ffi::ngx_conf_t) -> *mut c_void {
        <T as Module>::create_main_conf(ConfRef::from_ptr(cf))
            .map_or_else(ptr::null_mut, |p| p.as_ptr().cast())
    }

    unsafe extern "C" fn init_main_conf(
        cf: *mut ffi::ngx_conf_t,
        conf: *mut c_void,
    ) -> *mut c_char {
        <T as Module>::init_main_conf(ConfRef::from_ptr(cf), &mut *conf.cast())
            .map_or(NGX_CONF_ERROR, |_| NGX_CONF_OK)
    }

    unsafe extern "C" fn create_srv_conf(cf: *mut ffi::ngx_conf_t) -> *mut c_void {
        <T as Module>::create_srv_conf(ConfRef::from_ptr(cf))
            .map_or_else(ptr::null_mut, |p| p.as_ptr().cast())
    }

    unsafe extern "C" fn merge_srv_conf(
        cf: *mut ffi::ngx_conf_t,
        prev: *mut c_void,
        conf: *mut c_void,
    ) -> *mut c_char {
        <T as Module>::merge_srv_conf(ConfRef::from_ptr(cf), &mut *prev.cast(), &*conf.cast())
            .map_or(NGX_CONF_ERROR, |_| NGX_CONF_OK)
    }

    unsafe extern "C" fn create_loc_conf(cf: *mut ffi::ngx_conf_t) -> *mut c_void {
        <T as Module>::create_loc_conf(ConfRef::from_ptr(cf))
            .map_or_else(ptr::null_mut, |p| p.as_ptr().cast())
    }

    unsafe extern "C" fn merge_loc_conf(
        cf: *mut ffi::ngx_conf_t,
        prev: *mut c_void,
        conf: *mut c_void,
    ) -> *mut c_char {
        <T as Module>::merge_loc_conf(ConfRef::from_ptr(cf), &mut *prev.cast(), &*conf.cast())
            .map_or(NGX_CONF_ERROR, |_| NGX_CONF_OK)
    }
}

pub trait Module {
    type Error: From<<Self::MainConf as Merge>::Error>
        + From<<Self::SrvConf as Merge>::Error>
        + From<<Self::LocConf as Merge>::Error>;
    type MainConf: Default + Merge;
    type SrvConf: Default + Merge;
    type LocConf: Default + Merge;

    fn preconfiguration(_cf: &ConfRef) -> Result<(), Code> {
        Ok(())
    }

    fn postconfiguration(_cf: &ConfRef) -> Result<(), Code> {
        Ok(())
    }

    fn create_main_conf(cf: &ConfRef) -> Option<NonNull<Self::MainConf>> {
        cf.pool().allocate(Self::MainConf::default())
    }

    fn init_main_conf(_cf: &ConfRef, _conf: &Self::MainConf) -> Result<(), Self::Error> {
        Ok(())
    }

    fn create_srv_conf(cf: &ConfRef) -> Option<NonNull<Self::SrvConf>> {
        cf.pool().allocate(Self::SrvConf::default())
    }

    fn merge_srv_conf(
        _cf: &ConfRef,
        prev: &mut Self::SrvConf,
        conf: &Self::SrvConf,
    ) -> Result<(), Self::Error> {
        prev.merge(conf).map_err(Self::Error::from)
    }

    fn create_loc_conf(cf: &ConfRef) -> Option<NonNull<Self::LocConf>> {
        cf.pool().allocate(Self::LocConf::default())
    }

    fn merge_loc_conf(
        _cf: &ConfRef,
        prev: &mut Self::LocConf,
        conf: &Self::LocConf,
    ) -> Result<(), Self::Error> {
        prev.merge(conf).map_err(Self::Error::from)
    }
}
