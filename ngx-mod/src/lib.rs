pub extern crate memoffset;
pub extern crate ngx_rt as rt;

pub use ::ngx_mod_derive::{Conf, Module};

pub mod core;
pub mod http;
mod merge;
mod module;

pub use self::merge::Merge;
pub use self::module::{Module, ModuleMetadata, UnsafeModule, UNSET_INDEX};
