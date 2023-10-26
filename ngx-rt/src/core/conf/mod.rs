use std::{ffi::c_char, ptr};

#[allow(clippy::module_inception)]
mod conf;
#[macro_use]
mod r#enum;
mod file;
mod log;
mod open_file;
mod unset;

pub use self::conf::{Conf, ConfExt, ConfRef, UnsafeConf};
pub use self::file::{ConfFile, ConfFileRef};
pub use self::open_file::{OpenFile, OpenFileRef};
pub use self::r#enum::values as enum_values;
pub use self::unset::{unset, Unset};

pub const NGX_CONF_OK: *mut c_char = ptr::null_mut();
pub const NGX_CONF_ERROR: *mut c_char = usize::MAX as *mut c_char;
