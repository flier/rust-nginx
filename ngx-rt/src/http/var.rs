use std::{
    ops::{Deref, DerefMut},
    ptr::{null_mut, NonNull},
    slice,
};

use bitflags::bitflags;
use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{
    core::{Code, ConfRef, Str},
    ffi, native_callback, never_drop, property, AsRawMut, AsRawRef, Error,
};

use super::RequestRef;

pub type RawVar = ffi::ngx_http_variable_t;

foreign_type! {
    pub unsafe type Var: Send {
        type CType = ffi::ngx_http_variable_t;

        fn drop = never_drop::<ffi::ngx_http_variable_t>;
    }
}

#[macro_export]
macro_rules! ngx_var {
    () => {
        $crate::ffi::ngx_http_variable_t {
            name: $crate::ngx_str!(),
            set_handler: None,
            get_handler: None,
            data: 0,
            flags: 0,
            index: 0,
        }
    };
    ($name:literal) => {
        $crate::ffi::ngx_http_variable_t {
            name: $crate::ngx_str!($name),
            set_handler: None,
            get_handler: None,
            data: 0,
            flags: 0,
            index: 0,
        }
    };
    ($name:literal , $( $tt:tt )*) => {{
        let mut var = $crate::ffi::ngx_http_variable_t {
            name: $crate::ngx_str!($name),
            set_handler: None,
            get_handler: None,
            data: 0,
            flags: 0,
            index: 0,
        };

        ngx_var!(var => $( $tt )*);

        var
    }};
    ($var:ident => ) => {};
    ($var:ident => get = $fn:ident) => {
        $var.get_handler = Some($fn);
    };
    ($var:ident => get = $fn:ident , $( $tt:tt )*) => {
        $var.get_handler = Some($fn);

        ngx_var!($var => $( $tt:tt )*);
    };
    ($var:ident => set = $fn:ident) => {
        $var.set_handler = Some($fn);
    };
    ($var:ident => set = $fn:ident , $( $tt:tt )*) => {
        $var.set_handler = Some($fn);

        ngx_var!($var => $( $tt:tt )*);
    };
    ($var:ident => data = $data:expr) => {
        $var.data = $data;
    };
    ($var:ident => data = $data:expr , $( $tt:tt )*) => {
        $var.data = $data;

        ngx_var!($var => $( $tt:tt )*);
    };
    ($var:ident => flags = $flags:expr) => {
        $var.flags = $flags;
    };
    ($var:ident => flags = $flags:expr , $( $tt:tt )*) => {
        $var.flags = $flags;

        ngx_var!($var => $( $tt:tt )*);
    };
    ($var:ident => index = $index:expr) => {
        $var.index = $index;
    };
    ($var:ident => index = $index:expr , $( $tt:tt )*) => {
        $var.index = $index;

        ngx_var!($var => $( $tt:tt )*);
    };
}

impl ConfRef {
    pub fn add_variables<I: IntoIterator<Item = RawVar>>(&self, vars: I) -> Result<(), Error> {
        for var in vars {
            let var = unsafe { VarRef::from_ptr(&var as *const _ as *mut _) };

            let v = self
                .add_variable(var.name().unwrap().to_str().unwrap(), var.flags())
                .ok_or(Error::OutOfMemory)?;

            v.get_handler = var.get_handler;
            v.set_handler = var.set_handler;
            v.data = var.data;
        }

        Ok(())
    }

    pub fn add_variable<S: AsRef<str>>(&self, name: S, flags: Flags) -> Option<&mut VarRef> {
        unsafe {
            let name = name.as_ref();
            let name = Str::from(name);
            let p = ffi::ngx_http_add_variable(
                self.as_ptr(),
                &name as *const _ as *mut _,
                flags.bits() as usize,
            );

            NonNull::new(p).map(|p| VarRef::from_ptr_mut(p.as_ptr()))
        }
    }

    pub fn get_variable_index<S: AsRef<str>>(&self, name: S) -> Option<usize> {
        unsafe {
            let name = name.as_ref();
            let name = Str::from(name);
            let idx = ffi::ngx_http_get_variable_index(self.as_ptr(), &name as *const _ as *mut _);

            if idx >= 0 {
                Some(idx as usize)
            } else {
                None
            }
        }
    }
}

impl RequestRef {
    pub fn get_indexed_variable(&self, idx: usize) -> Option<&mut ValueRef> {
        unsafe {
            NonNull::new(ffi::ngx_http_get_indexed_variable(self.as_ptr(), idx))
                .map(|p| ValueRef::from_ptr_mut(p.as_ptr()))
        }
    }

    pub fn get_flushed_variable(&self, idx: usize) -> Option<&mut ValueRef> {
        unsafe {
            NonNull::new(ffi::ngx_http_get_flushed_variable(self.as_ptr(), idx))
                .map(|p| ValueRef::from_ptr_mut(p.as_ptr()))
        }
    }

    pub fn get_variable<S: AsRef<str>>(&self, name: S, key: usize) -> Option<&mut ValueRef> {
        unsafe {
            let name = name.as_ref();
            let name = Str::from(name);

            NonNull::new(ffi::ngx_http_get_variable(
                self.as_ptr(),
                &name as *const _ as *mut _,
                key,
            ))
            .map(|p| ValueRef::from_ptr_mut(p.as_ptr()))
        }
    }
}

impl Deref for VarRef {
    type Target = <Self as ForeignTypeRef>::CType;

    fn deref(&self) -> &Self::Target {
        unsafe { self.as_raw() }
    }
}

impl DerefMut for VarRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.as_raw_mut() }
    }
}

impl VarRef {
    property! {
        name: Str;
        data: usize;
        index: usize;
    }

    callback! {
        set_handler: SetVariableFn;
        get_handler: GetVariableFn;
    }

    pub fn flags(&self) -> Flags {
        Flags::from_bits_truncate(unsafe { self.as_raw().flags as u32 })
    }
}

bitflags! {
    pub struct Flags: u32 {
        const CHANGEABLE = ffi::NGX_HTTP_VAR_CHANGEABLE;
        const NO_CACHEABLE = ffi::NGX_HTTP_VAR_NOCACHEABLE;
        const INDEXED = ffi::NGX_HTTP_VAR_INDEXED;
        const NO_HASH = ffi::NGX_HTTP_VAR_NOHASH;
        const WEAK = ffi::NGX_HTTP_VAR_WEAK;
        const PREFIX = ffi::NGX_HTTP_VAR_PREFIX;
    }
}

#[native_callback]
pub type SetVariableFn = fn(req: &RequestRef, val: &ValueRef, data: usize);

#[native_callback]
pub type GetVariableFn = fn(req: &RequestRef, val: &ValueRef, data: usize) -> Result<(), Code>;

foreign_type! {
    pub unsafe type Value: Send {
        type CType = ffi::ngx_variable_value_t;

        fn drop = never_drop::<ffi::ngx_variable_value_t>;
    }
}

impl Deref for ValueRef {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl DerefMut for ValueRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_bytes_mut()
    }
}

pub fn null_value() -> &'static ValueRef {
    unsafe { ValueRef::from_ptr(&ffi::ngx_http_variable_null_value as *const _ as *mut _) }
}

pub fn true_value() -> &'static ValueRef {
    unsafe { ValueRef::from_ptr(&ffi::ngx_http_variable_true_value as *const _ as *mut _) }
}

impl ValueRef {
    property! {
        len(): u32 { get; set; }
    }

    flag! {
        valid { get; set; };
        no_cacheable { get; set; };
        not_found { get; set; };
        escape { get; set; };
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn data<T>(&self) -> Option<NonNull<T>> {
        NonNull::new(unsafe { self.as_raw().data.cast() })
    }

    pub fn set_data<T>(&mut self, data: Option<NonNull<T>>) -> &mut Self {
        unsafe { self.as_raw_mut().data = data.map_or_else(null_mut, |p| p.as_ptr().cast()) };

        self
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let r = self.as_raw();

            slice::from_raw_parts(r.data as *const _, r.len() as usize)
        }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            let r = self.as_raw();

            slice::from_raw_parts_mut(r.data, r.len() as usize)
        }
    }
}