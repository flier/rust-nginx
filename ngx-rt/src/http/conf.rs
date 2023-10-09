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
    unsafe fn main_conf<T>(&self, idx: usize) -> Option<&mut T>;
}

pub trait UnsafeSrvConf {
    /// Get the server configuration from context.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `srv_conf` array.
    unsafe fn srv_conf<T>(&self, idx: usize) -> Option<&mut T>;
}

pub trait UnsafeLocConf {
    /// Get the location configuration from context.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `loc_conf` array.
    unsafe fn loc_conf<T>(&self, idx: usize) -> Option<&mut T>;
}

impl UnsafeMainConf for ContextRef {
    unsafe fn main_conf<T>(&self, idx: usize) -> Option<&mut T> {
        NonNull::new(self.as_raw().main_conf.add(idx).read()).map(|p| p.cast::<T>().as_mut())
    }
}

impl UnsafeSrvConf for ContextRef {
    unsafe fn srv_conf<T>(&self, idx: usize) -> Option<&mut T> {
        NonNull::new(self.as_raw().srv_conf.add(idx).read()).map(|p| p.cast::<T>().as_mut())
    }
}

impl UnsafeLocConf for ContextRef {
    unsafe fn loc_conf<T>(&self, idx: usize) -> Option<&mut T> {
        NonNull::new(self.as_raw().loc_conf.add(idx).read()).map(|p| p.cast::<T>().as_mut())
    }
}

pub trait MainConfFor {
    /// Get the main configuration for the module.
    fn main_conf_for<T>(&self, m: &ModuleRef) -> Option<&mut T>;
}

pub trait SrvConfFor {
    /// Get the server configuration for the module.
    fn srv_conf_for<T>(&self, m: &ModuleRef) -> Option<&mut T>;
}

pub trait LocConfFor {
    /// Get the location configuration for the module.
    fn loc_conf_for<T>(&self, m: &ModuleRef) -> Option<&mut T>;
}

impl<M> MainConfFor for M
where
    M: UnsafeMainConf,
{
    fn main_conf_for<T>(&self, m: &ModuleRef) -> Option<&mut T> {
        unsafe { self.main_conf(m.context_index()) }
    }
}

impl<M> SrvConfFor for M
where
    M: UnsafeSrvConf,
{
    fn srv_conf_for<T>(&self, m: &ModuleRef) -> Option<&mut T> {
        unsafe { self.srv_conf(m.context_index()) }
    }
}

impl<M> LocConfFor for M
where
    M: UnsafeLocConf,
{
    fn loc_conf_for<T>(&self, m: &ModuleRef) -> Option<&mut T> {
        unsafe { self.loc_conf(m.context_index()) }
    }
}
