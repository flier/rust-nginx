use std::{
    mem::{self, MaybeUninit},
    ops::{Deref, DerefMut},
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

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.as_raw_mut().elts.cast(), self.len()) }
    }

    pub fn reserve(&mut self) -> Option<&mut MaybeUninit<T>> {
        unsafe {
            let p = ffi::ngx_array_push(self.as_ptr());

            if p.is_null() {
                None
            } else {
                Some(&mut *p.cast())
            }
        }
    }

    pub fn reserve_n(&mut self, n: usize) -> Option<&mut [MaybeUninit<T>]> {
        unsafe {
            let p = ffi::ngx_array_push_n(self.as_ptr(), n);

            if p.is_null() {
                None
            } else {
                Some(slice::from_raw_parts_mut(p.cast(), n))
            }
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

pub fn new<T: Sized>(p: &PoolRef, n: usize) -> Option<Array<T>> {
    let p = unsafe { ffi::ngx_array_create(p.as_ptr(), n, mem::size_of::<T>()) };

    if p.is_null() {
        None
    } else {
        Some(unsafe { Array::from_ptr(p) })
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Log, Pool};

    use super::*;

    #[test]
    fn array() {
        let p = Pool::new(4096, Log::stderr()).unwrap();
        let mut a = new::<usize>(&p, 4).unwrap();

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
