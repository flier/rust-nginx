use foreign_types::ForeignTypeRef;

use crate::{
    core::ModuleRef,
    ffi,
    http::{
        core::{LocConfRef, MainConfRef, SrvConfRef},
        LocConfFor, MainConfFor, SrvConfFor,
    },
};

pub fn module() -> &'static ModuleRef {
    unsafe { ModuleRef::from_ptr(&mut ffi::ngx_http_core_module as *mut _) }
}

pub fn main_conf<T>(cf: &T) -> &mut MainConfRef
where
    T: MainConfFor,
{
    cf.main_conf_for(module())
}

pub fn srv_conf<T>(cf: &T) -> &mut SrvConfRef
where
    T: SrvConfFor,
{
    cf.srv_conf_for(module())
}

pub fn loc_conf<T>(cf: &T) -> &mut LocConfRef
where
    T: LocConfFor,
{
    cf.loc_conf_for(module())
}
