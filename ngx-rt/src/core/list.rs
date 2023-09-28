use std::{
    mem::{self, MaybeUninit},
    ptr::NonNull,
    slice,
};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{ffi, never_drop, raw::FromRawMut, AsRawRef, FromRawRef};

use super::PoolRef;

foreign_type! {
    pub unsafe type List<T>: Send {
        type CType = ffi::ngx_list_t;
        type PhantomData = T;

        fn drop = never_drop::<ffi::ngx_list_t>;
    }
}

impl<T: Sized> List<T> {
    pub fn create(p: &PoolRef, n: usize) -> Option<&mut ListRef<T>> {
        unsafe { ListRef::from_raw_mut(ffi::ngx_list_create(p.as_ptr(), n, mem::size_of::<T>())) }
    }
}

impl<T: Sized> ListRef<T> {
    pub fn is_empty(&self) -> bool {
        unsafe {
            let r = self.as_raw();

            r.part.next.is_null() && r.part.nelts == 0
        }
    }

    pub fn len(&self) -> usize {
        self.parts().map(|p| p.len()).sum()
    }

    pub fn pool(&self) -> &PoolRef {
        unsafe { PoolRef::from_ptr(self.as_raw().pool) }
    }

    pub fn push(&mut self, value: T) -> Option<&mut T> {
        self.reserve().map(|p| p.write(value))
    }

    pub fn reserve(&mut self) -> Option<&mut MaybeUninit<T>> {
        unsafe { NonNull::new(ffi::ngx_list_push(self.as_ptr())).map(|p| p.cast().as_mut()) }
    }

    pub fn parts(&self) -> Parts<T> {
        Parts(Some(unsafe {
            PartRef::from_ptr(&self.as_raw().part as *const _ as *mut _)
        }))
    }

    pub fn iter(&self) -> Iter<T> {
        Iter(
            Some(unsafe { PartRef::from_ptr(&self.as_raw().part as *const _ as *mut _) }),
            0,
        )
    }
}

impl<T: Sized> Extend<T> for ListRef<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.push(value).unwrap();
        }
    }
}

pub struct Parts<'a, T>(Option<&'a PartRef<T>>);

impl<'a, T> Iterator for Parts<'a, T> {
    type Item = &'a PartRef<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(p) = self.0.take() {
            self.0 = p.next();

            Some(p)
        } else {
            None
        }
    }
}

impl<'a, T> IntoIterator for &'a ListRef<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Iter<'a, T>(Option<&'a PartRef<T>>, usize);

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while self.0.is_some() {
            if matches!(self.0, Some(p) if p.len() == self.1) {
                self.0 = self.0.take().unwrap().next();
            } else {
                let idx = self.1;

                self.1 += 1;

                return self.0.unwrap().get(idx);
            }
        }

        None
    }
}

foreign_type! {
    pub unsafe type Part<T>: Send {
        type CType = ffi::ngx_list_part_t;
        type PhantomData = T;

        fn drop = never_drop::<ffi::ngx_list_part_t>;
    }
}

impl<T> PartRef<T> {
    pub fn len(&self) -> usize {
        unsafe { self.as_raw().nelts }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        unsafe {
            let r = self.as_raw();

            if idx >= r.nelts {
                None
            } else {
                r.elts.cast::<T>().add(idx).as_ref()
            }
        }
    }

    pub fn next(&self) -> Option<&Self> {
        unsafe { Self::from_raw(self.as_raw().next.cast()) }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let r = self.as_raw();

            slice::from_raw_parts(r.elts as *const _ as *const _, r.nelts)
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            let r = self.as_raw();

            slice::from_raw_parts_mut(r.elts.cast(), r.nelts)
        }
    }
}

impl<T> AsRef<[T]> for PartRef<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> AsMut<[T]> for PartRef<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Log, Pool};

    use super::*;

    #[test]
    fn list() {
        let p = Pool::new(4096, Log::stderr()).unwrap();
        let l = List::create(&p, 4).unwrap();

        assert!(l.is_empty());
        assert_eq!(l.len(), 0);

        assert!(l.parts().next().is_some());
        assert!(l.parts().next().unwrap().next().is_none());

        assert_eq!(l.push(123), Some(&mut 123));
        assert_eq!(l.len(), 1);

        l.extend([456, 789]);
        assert_eq!(l.len(), 3);

        assert_eq!(l.iter().cloned().collect::<Vec<_>>(), vec![123, 456, 789]);
    }
}
