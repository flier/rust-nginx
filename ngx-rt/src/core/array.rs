use core::fmt;
use std::{
    mem::{self, MaybeUninit},
    ops::{Deref, DerefMut},
    ptr::NonNull,
    slice,
};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{ffi, AsRawMut, AsRawRef, FromRaw};

use super::PoolRef;

foreign_type! {
    pub unsafe type Array<T>: Send {
        type CType = ffi::ngx_array_t;
        type PhantomData = T;

        fn drop = ffi::ngx_array_destroy;
    }
}

impl<T: Sized> Array<T> {
    pub fn create(p: &PoolRef, n: usize) -> Option<Self> {
        unsafe { Self::from_raw(ffi::ngx_array_create(p.as_ptr(), n, mem::size_of::<T>())) }
    }
}

impl<T: Sized> ArrayRef<T> {
    pub fn init(&mut self, p: &PoolRef, n: usize) -> Option<&mut Self> {
        unsafe {
            let r = self.as_raw_mut();

            r.nelts = 0;
            r.size = mem::size_of::<T>();
            r.nalloc = n;
            r.pool = p.as_ptr();
            r.elts = p.palloc(n * mem::size_of::<T>());

            if r.elts.is_null() {
                None
            } else {
                Some(self)
            }
        }
    }

    pub fn is_null(&self) -> bool {
        unsafe { self.as_raw().elts.is_null() }
    }

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

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.as_raw_mut().elts.cast(), self.len()) }
    }

    pub fn reserve(&mut self) -> Option<&mut MaybeUninit<T>> {
        unsafe { NonNull::new(ffi::ngx_array_push(self.as_ptr())).map(|p| p.cast().as_mut()) }
    }

    pub fn reserve_n(&mut self, n: usize) -> Option<&mut [MaybeUninit<T>]> {
        unsafe {
            NonNull::new(ffi::ngx_array_push_n(self.as_ptr(), n))
                .map(|p| slice::from_raw_parts_mut(p.cast().as_ptr(), n))
        }
    }

    pub fn push(&mut self, value: T) -> Option<&mut T> {
        self.reserve().map(|p| p.write(value))
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

impl<T: Sized> ArrayRef<T> {
    pub fn extend<I>(&mut self, values: I) -> Option<&mut [T]>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let iter = values.into_iter();

        self.reserve_n(iter.len()).map(|s| {
            for (dst, src) in s.iter_mut().zip(iter) {
                dst.write(src);
            }

            unsafe { slice_assume_init_mut(s) }
        })
    }
}

impl<T: fmt::Debug> fmt::Debug for ArrayRef<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_list().entries(self.iter()).finish()
    }
}

unsafe fn slice_assume_init_mut<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    // SAFETY: similar to safety notes for `slice_get_ref`, but we have a
    // mutable reference which is also guaranteed to be valid for writes.
    &mut *(slice as *mut [MaybeUninit<T>] as *mut [T])
}

impl<T: Sized> Extend<T> for ArrayRef<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for value in iter {
            if let Some(p) = self.reserve() {
                p.write(value);
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Log, Pool};

    use super::*;

    #[test]
    fn array() {
        let p = Pool::new(4096, Log::stderr()).unwrap();
        let mut a = Array::<usize>::create(&p, 4).unwrap();

        assert_eq!(a.len(), 0);
        assert_eq!(a.cap(), 4);
        assert!(a.is_empty());
        assert!(!a.is_full());

        assert_eq!(a.push(1).unwrap(), &1);
        assert_eq!(a.extend([2, 3, 4]).unwrap(), &[2, 3, 4]);

        assert_eq!(a.len(), 4);
        assert_eq!(a.cap(), 4);
        assert!(!a.is_empty());
        assert!(a.is_full());

        assert_eq!(a.as_slice(), &[1, 2, 3, 4]);
    }
}
