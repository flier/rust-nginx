use std::{
    ffi::{c_char, c_void},
    ptr,
};

use foreign_types::ForeignTypeRef;

use crate::{
    rt::{
        core::{Code, ConfRef, NGX_CONF_ERROR, NGX_CONF_OK},
        ffi, http,
    },
    Merge,
};

pub trait UnsafeModule {
    /// A pre-configuration callback
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe extern "C" fn preconfiguration(cf: *mut ffi::ngx_conf_t) -> ffi::ngx_int_t;

    /// A post-configuration callback
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe extern "C" fn postconfiguration(cf: *mut ffi::ngx_conf_t) -> ffi::ngx_int_t;

    /// A callback for allocations and initializations of configurations for the main block configuration
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe extern "C" fn create_main_conf(cf: *mut ffi::ngx_conf_t) -> *mut c_void;

    /// A callback to set the configuration based on the directives supplied in the configuration files
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe extern "C" fn init_main_conf(cf: *mut ffi::ngx_conf_t, conf: *mut c_void)
        -> *mut c_char;

    /// A callback for allocations and initializations of configurations for the server block configuration
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe extern "C" fn create_srv_conf(cf: *mut ffi::ngx_conf_t) -> *mut c_void;

    /// A callback to merge the server block configuration with the main block
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe extern "C" fn merge_srv_conf(
        cf: *mut ffi::ngx_conf_t,
        prev: *mut c_void,
        conf: *mut c_void,
    ) -> *mut c_char;

    /// A callback for allocations and initializations of configurations for the location block configuration
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe extern "C" fn create_loc_conf(cf: *mut ffi::ngx_conf_t) -> *mut c_void;

    /// A callback to merge the location block configuration with the server block
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe extern "C" fn merge_loc_conf(
        cf: *mut ffi::ngx_conf_t,
        prev: *mut c_void,
        conf: *mut c_void,
    ) -> *mut c_char;
}

impl<T: Module> UnsafeModule for T {
    unsafe extern "C" fn preconfiguration(cf: *mut ffi::ngx_conf_t) -> ffi::ngx_int_t {
        <T as Module>::preconfiguration(ConfRef::from_ptr(cf))
            .err()
            .unwrap_or(Code::OK)
            .into()
    }

    unsafe extern "C" fn postconfiguration(cf: *mut ffi::ngx_conf_t) -> ffi::ngx_int_t {
        <T as Module>::postconfiguration(ConfRef::from_ptr(cf))
            .err()
            .unwrap_or(Code::OK)
            .into()
    }

    unsafe extern "C" fn create_main_conf(cf: *mut ffi::ngx_conf_t) -> *mut c_void {
        <T as Module>::create_main_conf(ConfRef::from_ptr(cf))
            .map_or_else(ptr::null_mut, |p| p as *mut _ as *mut _)
    }

    unsafe extern "C" fn init_main_conf(
        cf: *mut ffi::ngx_conf_t,
        conf: *mut c_void,
    ) -> *mut c_char {
        <T as Module>::init_main_conf(ConfRef::from_ptr(cf), &*conf.cast())
            .map_or(NGX_CONF_ERROR, |_| NGX_CONF_OK)
    }

    unsafe extern "C" fn create_srv_conf(cf: *mut ffi::ngx_conf_t) -> *mut c_void {
        <T as Module>::create_srv_conf(ConfRef::from_ptr(cf))
            .map_or_else(ptr::null_mut, |p| p as *mut _ as *mut _)
    }

    unsafe extern "C" fn merge_srv_conf(
        cf: *mut ffi::ngx_conf_t,
        prev: *mut c_void,
        conf: *mut c_void,
    ) -> *mut c_char {
        <T as Module>::merge_srv_conf(ConfRef::from_ptr(cf), &*prev.cast(), &mut *conf.cast())
            .map_or(NGX_CONF_ERROR, |_| NGX_CONF_OK)
    }

    unsafe extern "C" fn create_loc_conf(cf: *mut ffi::ngx_conf_t) -> *mut c_void {
        <T as Module>::create_loc_conf(ConfRef::from_ptr(cf))
            .map_or_else(ptr::null_mut, |p| p as *mut _ as *mut _)
    }

    unsafe extern "C" fn merge_loc_conf(
        cf: *mut ffi::ngx_conf_t,
        prev: *mut c_void,
        conf: *mut c_void,
    ) -> *mut c_char {
        <T as Module>::merge_loc_conf(ConfRef::from_ptr(cf), &*prev.cast(), &mut *conf.cast())
            .map_or(NGX_CONF_ERROR, |_| NGX_CONF_OK)
    }
}

pub trait Module: crate::Module {
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

    fn create_main_conf(cf: &ConfRef) -> Option<&mut Self::MainConf> {
        cf.pool().allocate_default()
    }

    fn init_main_conf(_cf: &ConfRef, _conf: &Self::MainConf) -> Result<(), Self::Error> {
        Ok(())
    }

    fn create_srv_conf(cf: &ConfRef) -> Option<&mut Self::SrvConf> {
        cf.pool().allocate_default()
    }

    fn merge_srv_conf(
        _cf: &ConfRef,
        prev: &Self::SrvConf,
        conf: &mut Self::SrvConf,
    ) -> Result<(), Self::Error> {
        conf.merge(prev).map_err(Self::Error::from)
    }

    fn create_loc_conf(cf: &ConfRef) -> Option<&mut Self::LocConf> {
        cf.pool().allocate_default()
    }

    fn merge_loc_conf(
        _cf: &ConfRef,
        prev: &Self::LocConf,
        conf: &mut Self::LocConf,
    ) -> Result<(), Self::Error> {
        conf.merge(prev).map_err(Self::Error::from)
    }

    fn main_conf<T>(cf: &T) -> &Self::MainConf
    where
        T: http::MainConf,
    {
        cf.main_conf(Self::module())
    }

    fn main_conf_mut<T>(cf: &T) -> &mut Self::MainConf
    where
        T: http::MainConf,
    {
        cf.main_conf_mut(Self::module())
    }

    fn srv_conf<T>(cf: &T) -> &Self::SrvConf
    where
        T: http::SrvConf,
    {
        cf.srv_conf(Self::module())
    }

    fn srv_conf_mut<T>(cf: &T) -> &mut Self::SrvConf
    where
        T: http::SrvConf,
    {
        cf.srv_conf_mut(Self::module())
    }

    fn loc_conf<T>(cf: &T) -> &Self::LocConf
    where
        T: http::LocConf,
    {
        cf.loc_conf(Self::module())
    }

    fn loc_conf_mut<T>(cf: &T) -> &mut Self::LocConf
    where
        T: http::LocConf,
    {
        cf.loc_conf_mut(Self::module())
    }
}
