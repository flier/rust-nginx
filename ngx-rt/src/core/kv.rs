use core::fmt;

use crate::{core::Str, ffi};

use foreign_types::ForeignTypeRef;

#[repr(transparent)]
#[derive(Clone)]
pub struct KeyValue(ffi::ngx_keyval_t);

impl KeyValue {
    pub fn key(&self) -> &Str {
        unsafe { Str::from_ptr(&self.0.key as *const _ as *mut _) }
    }

    pub fn value(&self) -> &Str {
        unsafe { Str::from_ptr(&self.0.value as *const _ as *mut _) }
    }
}

impl fmt::Debug for KeyValue {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("KeyValue")
            .field("key", self.key())
            .field("value", self.value())
            .finish()
    }
}

impl From<(Str, Str)> for KeyValue {
    fn from((key, value): (Str, Str)) -> Self {
        Self(ffi::ngx_keyval_t {
            key: key.into(),
            value: value.into(),
        })
    }
}

impl From<(&Str, &Str)> for KeyValue {
    fn from((key, value): (&Str, &Str)) -> Self {
        Self(ffi::ngx_keyval_t {
            key: key.into(),
            value: value.into(),
        })
    }
}

impl From<KeyValue> for (Str, Str) {
    fn from(kv: KeyValue) -> (Str, Str) {
        (*kv.key(), *kv.value())
    }
}
