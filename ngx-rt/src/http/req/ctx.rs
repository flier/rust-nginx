use std::ptr::NonNull;

use crate::{core::ModuleRef, AsRawRef};

use super::RequestRef;

pub trait ContextFor {
    /// Returns the module's context
    fn module_ctx_for<T>(&self, m: &ModuleRef) -> Option<&mut T>;
}

impl<M> ContextFor for M
where
    M: UnsafeContext,
{
    fn module_ctx_for<T>(&self, m: &ModuleRef) -> Option<&mut T> {
        unsafe { self.module_ctx(m.ctx_index()) }
    }
}

pub trait UnsafeContext {
    /// Returns the module's context
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `ctx` array.
    unsafe fn module_ctx<T>(&self, idx: usize) -> Option<&mut T>;

    /// Sets the module's context
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `ctx` array.
    unsafe fn set_module_ctx<T>(&self, idx: usize, ctx: NonNull<T>);
}

impl UnsafeContext for RequestRef {
    unsafe fn module_ctx<T>(&self, idx: usize) -> Option<&mut T> {
        NonNull::new(self.as_raw().ctx.add(idx).read()).map(|p| p.cast::<T>().as_mut())
    }

    unsafe fn set_module_ctx<T>(&self, idx: usize, ctx: NonNull<T>) {
        self.as_raw().ctx.add(idx).write(ctx.as_ptr().cast());
    }
}
