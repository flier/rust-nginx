use std::ptr::NonNull;

use crate::{core::ModuleRef, AsRawRef};

use super::RequestRef;

pub trait ModuleContext {
    /// Returns the module's context
    fn module_ctx<T>(&self, m: &ModuleRef) -> &T;

    /// Returns the module's context
    #[allow(clippy::mut_from_ref)]
    fn module_ctx_mut<T>(&self, m: &ModuleRef) -> &mut T;
}

impl<M> ModuleContext for M
where
    M: UnsafeModuleContext,
{
    fn module_ctx<T>(&self, m: &ModuleRef) -> &T {
        unsafe { self.unchecked_module_ctx(m.ctx_index()).as_ref() }
    }

    fn module_ctx_mut<T>(&self, m: &ModuleRef) -> &mut T {
        unsafe { self.unchecked_module_ctx(m.ctx_index()).as_mut() }
    }
}

pub trait UnsafeModuleContext {
    /// Returns the module's context
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `ctx` array.
    unsafe fn unchecked_module_ctx<T>(&self, idx: usize) -> NonNull<T>;

    /// Sets the module's context
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `ctx` array.
    unsafe fn unchecked_set_module_ctx<T>(&mut self, idx: usize, ctx: NonNull<T>);
}

impl UnsafeModuleContext for RequestRef {
    unsafe fn unchecked_module_ctx<T>(&self, idx: usize) -> NonNull<T> {
        NonNull::new(self.as_raw().ctx.add(idx).read().cast()).expect("ctx")
    }

    unsafe fn unchecked_set_module_ctx<T>(&mut self, idx: usize, ctx: NonNull<T>) {
        self.as_raw().ctx.add(idx).write(ctx.as_ptr().cast());
    }
}
