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

pub const fn unset<T: Unset>() -> T {
    T::UNSET
}

pub trait Unset {
    const UNSET: Self;

    fn is_unset(&self) -> bool
    where
        Self: Sized + PartialEq,
    {
        *self == Self::UNSET
    }

    fn get_or_set(&mut self, value: Self) -> &mut Self
    where
        Self: Sized + PartialEq,
    {
        if self.is_unset() {
            *self = value;
        }

        self
    }

    fn get_or_set_default(&mut self) -> &mut Self
    where
        Self: Sized + PartialEq + Default,
    {
        if self.is_unset() {
            *self = Self::default();
        }

        self
    }

    fn get_or_set_with<F>(&mut self, f: F) -> &mut Self
    where
        Self: Sized + PartialEq,
        F: FnOnce() -> Self,
    {
        if self.is_unset() {
            *self = f();
        }

        self
    }
}

impl Unset for u32 {
    const UNSET: Self = Self::MAX;
}

impl Unset for usize {
    const UNSET: Self = Self::MAX;
}

impl<T> Unset for *mut T {
    const UNSET: Self = usize::MAX as Self;
}
