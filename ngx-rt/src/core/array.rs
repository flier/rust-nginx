use std::{
    mem,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    slice,
};

use foreign_types::{foreign_type, ForeignType, ForeignTypeRef};

use crate::{ffi, AsRaw};

use super::PoolRef;

foreign_type! {
    pub unsafe type Array<T>: Send {
        type CType = ffi::ngx_array_t;
        type PhantomData = T;

        fn drop = ffi::ngx_array_destroy;
    }
}

impl<T: Sized> ArrayRef<T> {
    pub fn len(&self) -> usize {
        unsafe { self.as_raw().nelts }
    }

    pub fn cap(&self) -> usize {
        unsafe { self.as_raw().nalloc }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.len() == self.cap()
    }

    pub fn pool(&self) -> &PoolRef {
        unsafe { PoolRef::from_ptr(self.as_raw().pool) }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.as_raw().elts as *const _ as *const _, self.len()) }
    }

    pub fn as_mut_slice(&self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.as_raw().elts as *mut _, self.len()) }
    }

    pub fn reserve(&self) -> Option<NonNull<T>> {
        unsafe { NonNull::new(ffi::ngx_array_push(self.as_ptr()).cast()) }
    }

    pub fn reserve_n(&self, n: usize) -> Option<&mut [T]> {
        unsafe {
            NonNull::new(ffi::ngx_array_push_n(self.as_ptr(), n).cast::<T>())
                .map(|p| slice::from_raw_parts_mut(p.as_ptr(), n))
        }
    }

    pub fn push(&self, value: T) -> Option<&mut T> {
        self.reserve().and_then(|mut p| unsafe {
            p.as_ptr().write(value);

            Some(p.as_mut())
        })
    }
}

impl<T: Sized> AsRef<ffi::ngx_array_t> for ArrayRef<T> {
    fn as_ref(&self) -> &ffi::ngx_array_t {
        unsafe { self.as_raw() }
    }
}

impl<T: Sized> AsRef<[T]> for ArrayRef<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: Sized> AsMut<[T]> for ArrayRef<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T: Sized> Deref for ArrayRef<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: Sized> DerefMut for ArrayRef<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T: Sized + Copy> ArrayRef<T> {
    pub fn push_n(&self, values: &[T]) -> Option<&mut [T]> {
        self.reserve_n(values.len()).and_then(|s| {
            s.copy_from_slice(values);

            Some(s)
        })
    }
}

impl PoolRef {
    pub fn create_array<T: Sized>(&self, n: usize) -> Option<Array<T>> {
        let p = unsafe { ffi::ngx_array_create(self.as_ptr(), n, mem::size_of::<T>()) };

        if p.is_null() {
            None
        } else {
            Some(unsafe { Array::from_ptr(p) })
        }
    }
}
