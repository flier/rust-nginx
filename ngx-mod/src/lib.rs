pub extern crate ngx_derive as derive;
pub extern crate ngx_rt as rt;
pub extern crate ngx_sys as ffi;

pub use self::derive::Module;

pub mod core;
pub mod http;
mod merge;

pub use self::merge::Merge;
