use foreign_types::{foreign_type, ForeignTypeRef};
use ngx_rt_derive::native_callback;

use crate::{core::ArrayRef, ffi, http::RequestRef, never_drop, AsRawMut, AsRawRef, Error};

foreign_type! {
    pub unsafe type MainConf: Send {
        type CType = ffi::ngx_http_core_main_conf_t;

        fn drop = never_drop::<ffi::ngx_http_core_main_conf_t>;
    }
}

impl MainConfRef {
    pub fn phases(&self, p: Phases) -> &PhaseRef {
        unsafe { PhaseRef::from_ptr(&self.as_raw().phases[p as usize] as *const _ as *mut _) }
    }

    pub fn phases_mut(&mut self, p: Phases) -> &mut PhaseRef {
        unsafe { PhaseRef::from_ptr_mut(&mut self.as_raw_mut().phases[p as usize] as *mut _) }
    }
}

foreign_type! {
    pub unsafe type Phase: Send {
        type CType = ffi::ngx_http_phase_t;

        fn drop = never_drop::<ffi::ngx_http_phase_t>;
    }
}

impl PhaseRef {
    pub fn handlers(&self) -> &ArrayRef<Option<HandlerFn>> {
        unsafe { ArrayRef::from_ptr(&self.as_raw().handlers as *const _ as *mut _) }
    }

    pub fn handlers_mut(&mut self) -> &mut ArrayRef<ffi::ngx_http_handler_pt> {
        unsafe { ArrayRef::from_ptr_mut(&mut self.as_raw_mut().handlers as *mut _) }
    }
}

#[native_callback]
pub type HandlerFn = fn(req: &RequestRef) -> Result<(), Error>;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Phases {
    PostRead = ffi::ngx_http_phases_NGX_HTTP_POST_READ_PHASE,

    ServerRewrite = ffi::ngx_http_phases_NGX_HTTP_SERVER_REWRITE_PHASE,

    FindConfig = ffi::ngx_http_phases_NGX_HTTP_FIND_CONFIG_PHASE,
    Rewrite = ffi::ngx_http_phases_NGX_HTTP_REWRITE_PHASE,
    PostRewrite = ffi::ngx_http_phases_NGX_HTTP_POST_REWRITE_PHASE,

    Preaccess = ffi::ngx_http_phases_NGX_HTTP_PREACCESS_PHASE,

    Access = ffi::ngx_http_phases_NGX_HTTP_ACCESS_PHASE,
    PostAccess = ffi::ngx_http_phases_NGX_HTTP_POST_ACCESS_PHASE,

    Precontent = ffi::ngx_http_phases_NGX_HTTP_PRECONTENT_PHASE,

    Content = ffi::ngx_http_phases_NGX_HTTP_CONTENT_PHASE,

    Log = ffi::ngx_http_phases_NGX_HTTP_LOG_PHASE,
}
