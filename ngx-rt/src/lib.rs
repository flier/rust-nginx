#[doc(hidden)]
pub extern crate foreign_types;
pub extern crate ngx_sys as ffi;

pub use ::ngx_rt_derive::{native_callback, native_handler, native_setter};

#[macro_use]
mod macros;

#[macro_use]
pub mod core;
mod error;
pub mod event;
mod raw;

#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "stream")]
pub mod stream;

pub use self::error::{Error, RawErr, RawOk, RawResult, Result};
pub use self::raw::{
    AsRawMut, AsRawRef, AsResult, FromRaw, FromRawMut, FromRawRef, NativeCallback,
};

#[doc(hidden)]
pub(crate) use self::raw::never_drop;
