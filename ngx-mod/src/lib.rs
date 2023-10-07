pub extern crate memoffset;
pub extern crate ngx_rt as rt;

pub use ::ngx_mod_derive::{Conf, Module};

#[macro_use]
pub mod conf;
pub mod core;
mod merge;
mod module;

#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "stream")]
pub mod stream;

pub use self::conf::UnsafeConf;
pub use self::merge::Merge;
pub use self::module::{Module, ModuleMetadata, UnsafeModule, UNSET_INDEX};
