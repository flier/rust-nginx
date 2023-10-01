pub extern crate foreign_types;
pub extern crate ngx_sys as ffi;

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
