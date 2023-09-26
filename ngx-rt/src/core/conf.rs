use foreign_types::{foreign_type, ForeignTypeRef};

use crate::ffi;

use super::{fake_drop, BufRef, CycleRef, LogRef, ModuleType, PoolRef};

foreign_type! {
    pub unsafe type Conf: Send {
        type CType = ffi::ngx_conf_t;

        fn drop = fake_drop::<ffi::ngx_conf_t>;
    }
}

impl ConfRef {
    pub fn cycle(&self) -> &CycleRef {
        unsafe { CycleRef::from_ptr(self.as_raw().cycle) }
    }

    pub fn pool(&self) -> &PoolRef {
        unsafe { PoolRef::from_ptr(self.as_raw().pool) }
    }

    pub fn temp_pool(&self) -> &PoolRef {
        unsafe { PoolRef::from_ptr(self.as_raw().temp_pool) }
    }

    pub fn conf_file(&self) -> &ConfFileRef {
        unsafe { ConfFileRef::from_ptr(self.as_raw().conf_file) }
    }

    pub fn log(&self) -> &LogRef {
        unsafe { LogRef::from_ptr(self.as_raw().log) }
    }

    pub fn module_type(&self) -> ModuleType {
        ModuleType::try_from(unsafe { self.as_raw().module_type as u32 }).expect("module_type")
    }

    unsafe fn as_raw(&self) -> &ffi::ngx_conf_t {
        &*self.as_ptr()
    }
}

foreign_type! {
    pub unsafe type ConfFile: Send {
        type CType = ffi::ngx_conf_file_t;

        fn drop = fake_drop::<ffi::ngx_conf_file_t>;
    }
}

impl ConfFileRef {
    pub fn buffer(&self) -> &BufRef {
        unsafe { BufRef::from_ptr(self.as_raw().buffer) }
    }

    pub fn dump(&self) -> &BufRef {
        unsafe { BufRef::from_ptr(self.as_raw().dump) }
    }

    pub fn line(&self) -> usize {
        unsafe { self.as_raw().line }
    }
    unsafe fn as_raw(&self) -> &ffi::ngx_conf_file_t {
        &*self.as_ptr()
    }
}
