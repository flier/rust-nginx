use const_zero::const_zero;

use crate::{core::Str, ffi};

pub const fn unset<T: Unset>() -> T {
    T::UNSET
}

pub trait Unset: Sized {
    const UNSET: Self;

    fn is_unset(&self) -> bool;

    fn or_insert(&mut self, value: Self) -> &mut Self {
        if self.is_unset() {
            *self = value;
        }

        self
    }

    fn or_insert_with<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce() -> Self,
    {
        if self.is_unset() {
            *self = f();
        }

        self
    }

    fn get_or_set(&mut self, value: Self) -> &mut Self {
        if self.is_unset() {
            *self = value;
        }

        self
    }

    fn get_or_set_default(&mut self) -> &mut Self
    where
        Self: Default,
    {
        if self.is_unset() {
            *self = Self::default();
        }

        self
    }

    fn get_or_set_with<F>(&mut self, f: F) -> &mut Self
    where
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

    fn is_unset(&self) -> bool {
        *self == Self::UNSET
    }
}

impl Unset for i32 {
    const UNSET: Self = u32::MAX as i32;

    fn is_unset(&self) -> bool {
        *self == Self::UNSET
    }
}

impl Unset for u64 {
    const UNSET: Self = Self::MAX;

    fn is_unset(&self) -> bool {
        *self == Self::UNSET
    }
}

impl Unset for i64 {
    const UNSET: Self = u64::MAX as i64;

    fn is_unset(&self) -> bool {
        *self == Self::UNSET
    }
}

impl Unset for usize {
    const UNSET: Self = Self::MAX;

    fn is_unset(&self) -> bool {
        *self == Self::UNSET
    }
}

impl Unset for isize {
    const UNSET: Self = usize::MAX as isize;

    fn is_unset(&self) -> bool {
        *self == Self::UNSET
    }
}

impl<T> Unset for *const T {
    const UNSET: Self = usize::MAX as Self;

    fn is_unset(&self) -> bool {
        *self == Self::UNSET || self.is_null()
    }
}

impl<T> Unset for *mut T {
    const UNSET: Self = usize::MAX as Self;

    fn is_unset(&self) -> bool {
        *self == Self::UNSET || self.is_null()
    }
}

impl<T> Unset for Option<T> {
    const UNSET: Self = None;

    fn is_unset(&self) -> bool {
        self.is_none()
    }
}

impl Unset for Str {
    const UNSET: Self = Self::null();

    fn is_unset(&self) -> bool {
        self.is_null()
    }
}

impl Unset for ffi::ngx_str_t {
    const UNSET: Self = crate::ngx_str!();

    fn is_unset(&self) -> bool {
        self.data.is_null()
    }
}

impl Unset for ffi::ngx_array_t {
    const UNSET: Self = unsafe { const_zero!(ffi::ngx_array_t) };

    fn is_unset(&self) -> bool {
        self.elts.is_null()
    }
}
