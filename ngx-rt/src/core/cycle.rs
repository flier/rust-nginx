use std::{ptr::NonNull, slice};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{ffi, never_drop, property, str, AsRawRef};

use super::{conf::OpenFile, shm, ConnList, ConnSlice, ListRef, LogRef, ModuleRef, PoolRef};

foreign_type! {
    pub unsafe type Cycle: Send {
        type CType = ffi::ngx_cycle_t;

        fn drop = never_drop::<ffi::ngx_cycle_t>;
    }
}

impl Cycle {
    pub fn current() -> &'static CycleRef {
        unsafe { CycleRef::from_ptr(ffi::ngx_cycle) }
    }
}

impl CycleRef {
    property! {
        /// Cycle pool.
        ///
        /// Created for each new cycle.
        pool: &PoolRef;

        /// Cycle log.
        ///
        /// Initially inherited from the old cycle, it is set to point to new_log after the configuration is read.
        log: &LogRef;

        /// Cycle log, created by the configuration.
        ///
        /// It's affected by the root-scope error_log directive.
        &new_log: &LogRef;

        old_cycle as &CycleRef;
    }

    pub fn conns(&self) -> ConnSlice {
        unsafe {
            let r = self.as_raw();

            ConnSlice(slice::from_raw_parts(r.connections, r.connection_n))
        }
    }

    /// currently available connections
    pub fn free_conns(&self) -> ConnList {
        unsafe {
            let r = self.as_raw();

            ConnList::new(NonNull::new(r.free_connections), r.free_connection_n)
        }
    }

    pub fn modules(&self) -> &[&ModuleRef] {
        unsafe {
            let r = self.as_raw();

            std::slice::from_raw_parts(r.modules.cast(), r.modules_n)
        }
    }

    /// open file objects
    pub fn open_files(&self) -> &ListRef<OpenFile> {
        unsafe { ListRef::from_ptr(&self.as_raw().open_files as *const _ as *mut _) }
    }

    /// hared memory zones
    pub fn shared_memory(&self) -> &ListRef<shm::Zone> {
        unsafe { ListRef::from_ptr(&self.as_raw().shared_memory as *const _ as *mut _) }
    }

    str! {
        &conf_file;
        &conf_param;
        &conf_prefix;
        &prefix;
        &error_log;
        &lock_file;
        &hostname;
    }
}

impl AsRef<LogRef> for CycleRef {
    fn as_ref(&self) -> &LogRef {
        self.log()
    }
}

pub trait UnsafeConfContext {
    /// Get the configuration context from cycle.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    /// The caller must ensure that `idx` is within the bounds of the `conf_ctx` array.
    unsafe fn unchecked_conf_ctx<T>(&self, idx: usize) -> Option<NonNull<T>>;
}

impl UnsafeConfContext for CycleRef {
    unsafe fn unchecked_conf_ctx<T>(&self, idx: usize) -> Option<NonNull<T>> {
        NonNull::new(self.as_raw().conf_ctx.add(idx).read().cast())
    }
}

pub trait ConfContext {
    /// Get the reference of configuration context from cycle.
    fn conf_ctx<T>(&self, m: &ModuleRef) -> Option<&T>;

    /// Get the mutable reference of configuration context from cycle.
    fn conf_ctx_mut<T>(&self, m: &ModuleRef) -> Option<&mut T>;
}

impl<C> ConfContext for C
where
    C: UnsafeConfContext,
{
    fn conf_ctx<T>(&self, m: &ModuleRef) -> Option<&T> {
        unsafe { self.unchecked_conf_ctx(m.ctx_index()).map(|p| p.as_ref()) }
    }

    fn conf_ctx_mut<T>(&self, m: &ModuleRef) -> Option<&mut T> {
        unsafe {
            self.unchecked_conf_ctx(m.ctx_index())
                .map(|mut p| p.as_mut())
        }
    }
}
