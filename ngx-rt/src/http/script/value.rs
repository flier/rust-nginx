use std::mem;

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{
    core::{ConfRef, PoolRef, Str},
    ffi, never_drop, AsResult, Error,
};

foreign_type! {
    pub unsafe type ComplexValue: Send {
        type CType = ffi::ngx_http_complex_value_t;

        fn drop = never_drop::<ffi::ngx_http_complex_value_t>;
    }
}

impl ComplexValue {
    pub fn alloc(p: &PoolRef) -> Option<&mut ComplexValueRef> {
        p.calloc::<ffi::ngx_http_complex_value_t>()
            .map(|p| unsafe { ComplexValueRef::from_ptr_mut(p as *mut _) })
    }
}

impl ComplexValueRef {
    pub fn compiler<'a>(&'a mut self, cf: &'a ConfRef, value: &'a str) -> Compiler<'a> {
        Compiler::new(cf, value, self)
    }
}

pub struct Compiler<'a> {
    pub cf: &'a ConfRef,
    pub value: &'a str,
    pub complex_value: &'a mut ComplexValueRef,
    pub zeroed: bool,
    pub conf_prefix: bool,
    pub root_prefix: bool,
}

impl<'a> Compiler<'a> {
    pub fn new(cf: &'a ConfRef, value: &'a str, complex_value: &'a mut ComplexValueRef) -> Self {
        Self {
            cf,
            value,
            complex_value,
            zeroed: false,
            conf_prefix: false,
            root_prefix: false,
        }
    }

    pub fn with_zeored(&mut self) -> &mut Self {
        self.zeroed = true;
        self
    }

    pub fn with_conf_prefix(&mut self) -> &mut Self {
        self.conf_prefix = true;
        self
    }

    pub fn with_root_prefix(&mut self) -> &mut Self {
        self.root_prefix = true;
        self
    }

    pub fn compile(self) -> Result<&'a mut ComplexValueRef, Error> {
        unsafe {
            let value = Str::from(self.value);
            let mut ccv = ffi::ngx_http_compile_complex_value_t {
                cf: self.cf.as_ptr(),
                value: &value as *const _ as *mut _,
                complex_value: self.complex_value.as_ptr(),
                ..mem::zeroed()
            };

            if self.zeroed {
                ccv.set_zero(1);
            }
            if self.conf_prefix {
                ccv.set_conf_prefix(1);
            }
            if self.root_prefix {
                ccv.set_root_prefix(1);
            }

            ffi::ngx_http_compile_complex_value(&mut ccv)
                .ok_or_else(Error::InternalError)
                .map(|_| self.complex_value)
        }
    }
}
