use std::{ffi::c_char, ptr};

#[allow(clippy::module_inception)]
mod conf;
mod file;
mod log;
mod open_file;

pub use self::conf::{Conf, ConfRef};
pub use self::file::{ConfFile, ConfFileRef};
pub use self::open_file::{OpenFile, OpenFileRef};

pub const NGX_CONF_OK: *mut c_char = ptr::null_mut();
pub const NGX_CONF_ERROR: *mut c_char = usize::MAX as *mut c_char;
