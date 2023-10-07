use crate::foreign_types::ForeignTypeRef;

use crate::{core::ModuleRef, ffi};

pub fn module() -> &'static ModuleRef {
    unsafe { ModuleRef::from_ptr(&mut ffi::ngx_http_upstream_module as *mut _) }
}
