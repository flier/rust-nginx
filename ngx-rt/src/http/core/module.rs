use foreign_types::ForeignTypeRef;

use crate::{
    core::ModuleRef,
    ffi,
    http::{
        core::{LocConfRef, MainConfRef, SrvConfRef},
        LocConf, MainConf, SrvConf,
    },
};

pub fn module() -> &'static ModuleRef {
    unsafe { ModuleRef::from_ptr(&mut ffi::ngx_http_core_module as *mut _) }
}

pub fn main_conf<T>(cf: &T) -> &MainConfRef
where
    T: MainConf,
{
    cf.main_conf(module())
}

pub fn main_conf_mut<T>(cf: &T) -> &mut MainConfRef
where
    T: MainConf,
{
    cf.main_conf_mut(module())
}

pub fn srv_conf<T>(cf: &T) -> &SrvConfRef
where
    T: SrvConf,
{
    cf.srv_conf(module())
}

pub fn srv_conf_mut<T>(cf: &T) -> &mut SrvConfRef
where
    T: SrvConf,
{
    cf.srv_conf_mut(module())
}

pub fn loc_conf<T>(cf: &T) -> &LocConfRef
where
    T: LocConf,
{
    cf.loc_conf(module())
}

pub fn loc_conf_mut<T>(cf: &T) -> &mut LocConfRef
where
    T: LocConf,
{
    cf.loc_conf_mut(module())
}
