use std::mem::{self, MaybeUninit};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{
    core::{ConfRef, PoolRef, Str},
    ffi,
    http::RequestRef,
    never_drop, AsResult, Error,
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
    pub fn evaluate(&self, req: &RequestRef) -> Result<Str, Error> {
        let res = Str::default();

        unsafe { ffi::ngx_http_complex_value(req.as_ptr(), self.as_ptr(), res.as_ptr()) }
            .ok()
            .map(|_| res)
            .map_err(|_| Error::OutOfMemory)
    }

    pub fn evaluate_size(&self, req: &RequestRef, default: usize) -> usize {
        unsafe { ffi::ngx_http_complex_value_size(req.as_ptr(), self.as_ptr(), default) }
    }
}

pub struct Compiler<'a> {
    /// Configuration pointer
    pub cf: &'a ConfRef,
    /// Flag that enables zero-terminating value
    pub zero: bool,
    /// Prefixes the result with the configuration prefix
    /// (the directory where nginx is currently looking for configuration)
    pub conf_prefix: bool,
    /// Prefixes the result with the root prefix
    /// (the normal nginx installation prefix)
    pub root_prefix: bool,
}

impl<'a> Compiler<'a> {
    pub fn new(cf: &'a ConfRef) -> Self {
        Self {
            cf,
            zero: false,
            conf_prefix: false,
            root_prefix: false,
        }
    }

    pub fn with_zero(&mut self) -> &mut Self {
        self.zero = true;
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

    pub fn compile(&self, s: &Str) -> Result<<ComplexValueRef as ForeignTypeRef>::CType, Error> {
        let mut v = MaybeUninit::<<ComplexValueRef as ForeignTypeRef>::CType>::uninit();

        unsafe {
            self.compile_to(s, ComplexValueRef::from_ptr_mut(v.as_mut_ptr()))
                .map(|_| v.assume_init())
        }
    }

    pub fn compile_to(&self, s: &Str, v: &mut ComplexValueRef) -> Result<(), Error> {
        unsafe {
            let mut ccv = ffi::ngx_http_compile_complex_value_t {
                cf: self.cf.as_ptr(),
                value: s.as_ptr(),
                complex_value: v.as_ptr(),
                ..mem::zeroed()
            };

            if self.zero {
                ccv.set_zero(1);
            }
            if self.conf_prefix {
                ccv.set_conf_prefix(1);
            }
            if self.root_prefix {
                ccv.set_root_prefix(1);
            }

            ffi::ngx_http_compile_complex_value(&mut ccv)
                .ok()
                .map(|_| ())
                .map_err(Error::InternalError)
        }
    }
}
