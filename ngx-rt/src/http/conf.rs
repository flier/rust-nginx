use std::ptr::NonNull;

use foreign_types::foreign_type;

use crate::{core::ModuleRef, ffi, never_drop, AsRawRef};

foreign_type! {
    pub unsafe type Context: Send {
        type CType = ffi::ngx_http_conf_ctx_t;

        fn drop = never_drop::<ffi::ngx_http_conf_ctx_t>;
    }
}

pub trait UnsafeMainConf {
    /// Get the main configuration from context.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `main_conf` array.
    unsafe fn unchecked_main_conf<T>(&self, idx: usize) -> Option<NonNull<T>>;
}

pub trait UnsafeSrvConf {
    /// Get the server configuration from context.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `srv_conf` array.
    unsafe fn unchecked_srv_conf<T>(&self, idx: usize) -> Option<NonNull<T>>;
}

pub trait UnsafeLocConf {
    /// Get the location configuration from context.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `loc_conf` array.
    unsafe fn unchecked_loc_conf<T>(&self, idx: usize) -> Option<NonNull<T>>;
}

impl UnsafeMainConf for ContextRef {
    unsafe fn unchecked_main_conf<T>(&self, idx: usize) -> Option<NonNull<T>> {
        NonNull::new(self.as_raw().main_conf.add(idx).read().cast())
    }
}

impl UnsafeSrvConf for ContextRef {
    unsafe fn unchecked_srv_conf<T>(&self, idx: usize) -> Option<NonNull<T>> {
        NonNull::new(self.as_raw().srv_conf.add(idx).read().cast())
    }
}

impl UnsafeLocConf for ContextRef {
    unsafe fn unchecked_loc_conf<T>(&self, idx: usize) -> Option<NonNull<T>> {
        NonNull::new(self.as_raw().loc_conf.add(idx).read().cast())
    }
}

pub trait MainConf {
    /// Get the main configuration for the module.
    fn main_conf<T>(&self, m: &ModuleRef) -> Option<&T>;

    /// Get the main configuration for the module.
    #[allow(clippy::mut_from_ref)]
    fn main_conf_mut<T>(&self, m: &ModuleRef) -> Option<&mut T>;
}

pub trait SrvConf {
    /// Get the server configuration for the module.
    fn srv_conf<T>(&self, m: &ModuleRef) -> Option<&T>;

    /// Get the server configuration for the module.
    #[allow(clippy::mut_from_ref)]
    fn srv_conf_mut<T>(&self, m: &ModuleRef) -> Option<&mut T>;
}

pub trait LocConf {
    /// Get the location configuration for the module.
    fn loc_conf<T>(&self, m: &ModuleRef) -> Option<&T>;

    /// Get the location configuration for the module.
    #[allow(clippy::mut_from_ref)]
    fn loc_conf_mut<T>(&self, m: &ModuleRef) -> Option<&mut T>;
}

impl<M> MainConf for M
where
    M: UnsafeMainConf,
{
    fn main_conf<T>(&self, m: &ModuleRef) -> Option<&T> {
        unsafe { self.unchecked_main_conf(m.ctx_index()).map(|p| p.as_ref()) }
    }

    fn main_conf_mut<T>(&self, m: &ModuleRef) -> Option<&mut T> {
        unsafe {
            self.unchecked_main_conf(m.ctx_index())
                .map(|mut p| p.as_mut())
        }
    }
}

impl<M> SrvConf for M
where
    M: UnsafeSrvConf,
{
    fn srv_conf<T>(&self, m: &ModuleRef) -> Option<&T> {
        unsafe { self.unchecked_srv_conf(m.ctx_index()).map(|p| p.as_ref()) }
    }

    fn srv_conf_mut<T>(&self, m: &ModuleRef) -> Option<&mut T> {
        unsafe {
            self.unchecked_srv_conf(m.ctx_index())
                .map(|mut p| p.as_mut())
        }
    }
}

impl<M> LocConf for M
where
    M: UnsafeLocConf,
{
    fn loc_conf<T>(&self, m: &ModuleRef) -> Option<&T> {
        unsafe { self.unchecked_loc_conf(m.ctx_index()).map(|p| p.as_ref()) }
    }

    fn loc_conf_mut<T>(&self, m: &ModuleRef) -> Option<&mut T> {
        unsafe {
            self.unchecked_loc_conf(m.ctx_index())
                .map(|mut p| p.as_mut())
        }
    }
}
