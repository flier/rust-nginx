pub extern crate foreign_types;
pub extern crate ngx_derive as derive;
pub extern crate ngx_sys as ffi;

pub use self::derive::{native_callback, native_handler};

pub mod core;
mod error;
pub mod event;
pub mod http;
mod raw;
#[macro_use]
mod macros;

pub use self::error::{Error, Result};
pub(crate) use self::raw::never_drop;
pub use self::raw::{AsRawMut, AsRawRef, AsResult, FromRaw, FromRawMut, FromRawRef};
