pub extern crate ngx_derive as derive;
pub extern crate ngx_rt as rt;
pub extern crate ngx_sys as ffi;

pub use self::derive::{native_handler, Module};

pub mod core;
pub mod http;
mod merge;
mod module;

pub use self::merge::Merge;
pub use self::module::{Module, ModuleMetadata, UnsafeModule, UNSET_INDEX};
