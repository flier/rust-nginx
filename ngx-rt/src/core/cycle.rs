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

impl CycleRef {
    property!(pool: &PoolRef);
    property!(log: &LogRef);
    property!(&new_log: &LogRef);

    pub fn conns(&self) -> ConnSlice {
        unsafe {
            let r = self.as_raw();

            ConnSlice(slice::from_raw_parts(r.connections, r.connection_n))
        }
    }

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

    pub fn open_files(&self) -> &ListRef<OpenFile> {
        unsafe { ListRef::from_ptr(&self.as_raw().open_files as *const _ as *mut _) }
    }

    pub fn shared_memory(&self) -> &ListRef<shm::Zone> {
        unsafe { ListRef::from_ptr(&self.as_raw().shared_memory as *const _ as *mut _) }
    }

    property!(old_cycle as &CycleRef);

    str!(conf_file);
    str!(conf_param);
    str!(conf_prefix);
    str!(prefix);
    str!(error_log);
    str!(lock_file);
    str!(hostname);
}
