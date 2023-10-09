use crate::foreign_types::ForeignTypeRef;

use crate::{
    core::ModuleRef,
    ffi,
    http::{
        upstream::{MainConfRef, SrvConfRef},
        MainConfFor, SrvConfFor,
    },
};

pub fn module() -> &'static ModuleRef {
    unsafe { ModuleRef::from_ptr(&mut ffi::ngx_http_upstream_module as *mut _) }
}

pub fn main_conf<T>(cf: &T) -> Option<&mut MainConfRef>
where
    T: MainConfFor,
{
    cf.main_conf_for(module())
}

pub fn srv_conf<T>(cf: &T) -> Option<&mut SrvConfRef>
where
    T: SrvConfFor,
{
    cf.srv_conf_for(module())
}
