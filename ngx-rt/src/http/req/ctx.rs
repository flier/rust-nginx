use std::ptr::NonNull;

use crate::{core::ModuleRef, AsRawRef};

use super::RequestRef;

pub trait ModuleContext {
    /// Returns the module's context
    fn module_ctx<T>(&self, m: &ModuleRef) -> Option<&T>;

    /// Returns the module's context
    #[allow(clippy::mut_from_ref)]
    fn module_ctx_mut<T>(&self, m: &ModuleRef) -> Option<&mut T>;

    /// Sets the module's context
    fn set_module_ctx<T>(&self, m: &ModuleRef, ctx: &T);
}

impl<M> ModuleContext for M
where
    M: UnsafeModuleContext,
{
    fn module_ctx<T>(&self, m: &ModuleRef) -> Option<&T> {
        unsafe { self.unchecked_module_ctx(m.ctx_index()).map(|p| p.as_ref()) }
    }

    fn module_ctx_mut<T>(&self, m: &ModuleRef) -> Option<&mut T> {
        unsafe {
            self.unchecked_module_ctx(m.ctx_index())
                .map(|mut p| p.as_mut())
        }
    }

    fn set_module_ctx<T>(&self, m: &ModuleRef, ctx: &T) {
        unsafe {
            self.unchecked_set_module_ctx(
                m.ctx_index(),
                NonNull::new_unchecked(ctx as *const _ as *mut T),
            );
        }
    }
}

pub trait UnsafeModuleContext {
    /// Returns the module's context
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `ctx` array.
    unsafe fn unchecked_module_ctx<T>(&self, idx: usize) -> Option<NonNull<T>>;

    /// Sets the module's context
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `ctx` array.
    unsafe fn unchecked_set_module_ctx<T>(&self, idx: usize, ctx: NonNull<T>);
}

impl UnsafeModuleContext for RequestRef {
    unsafe fn unchecked_module_ctx<T>(&self, idx: usize) -> Option<NonNull<T>> {
        NonNull::new(self.as_raw().ctx.add(idx).read().cast())
    }

    unsafe fn unchecked_set_module_ctx<T>(&self, idx: usize, ctx: NonNull<T>) {
        self.as_raw().ctx.add(idx).write(ctx.as_ptr().cast());
    }
}
