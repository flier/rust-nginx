use foreign_types::ForeignTypeRef;

use crate::{
    core::{ConfContext, CycleRef, ModuleRef},
    ffi,
};

use super::{ConfContextRef, MainConf};

pub fn module() -> &'static ModuleRef {
    unsafe { ModuleRef::from_ptr(&mut ffi::ngx_http_module as *mut _) }
}

pub fn conf_ctx(cycle: &CycleRef) -> Option<&ConfContextRef> {
    cycle.conf_ctx(module())
}

pub fn main_conf<'a, T>(cycle: &'a CycleRef, m: &ModuleRef) -> Option<&'a T> {
    conf_ctx(cycle).and_then(|ctx| ctx.main_conf(m))
}
