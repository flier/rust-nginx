pub extern crate memoffset;
pub extern crate ngx_rt as rt;

pub use ::ngx_mod_derive::{Conf, Module};

pub mod core;
mod merge;
mod module;

#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "stream")]
pub mod stream;

pub use self::merge::Merge;
pub use self::module::{Module, ModuleMetadata, UnsafeModule, UNSET_INDEX};
