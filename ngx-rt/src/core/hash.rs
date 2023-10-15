use std::{ptr::NonNull, slice, str};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{core::Str, ffi, never_drop, AsRawRef, FromRawRef};

pub fn key<T: AsRef<[u8]>>(data: T) -> usize {
    let data = data.as_ref();

    unsafe { ffi::ngx_hash_key(data.as_ptr() as *mut _, data.len()) }
}

pub fn key_lc<T: AsRef<[u8]>>(data: T) -> usize {
    let data = data.as_ref();

    unsafe { ffi::ngx_hash_key_lc(data.as_ptr() as *mut _, data.len()) }
}

/// Generate a lowercase hash key.
///
/// # Safety
///
/// The caller has provided a valid `dst` that points to range of bytes of at least `src.len()` bytes.
pub unsafe fn unchecked_strlow<D: AsMut<[u8]>, S: AsRef<[u8]>>(mut dst: D, src: S) -> usize {
    let dst = dst.as_mut();
    let src = src.as_ref();

    assert!(dst.len() >= src.len());

    ffi::ngx_hash_strlow(dst.as_mut_ptr(), src.as_ptr() as *mut _, src.len())
}

pub fn strlow_in_place<T: AsMut<[u8]>>(mut data: T) -> usize {
    let data = data.as_mut();

    unsafe { ffi::ngx_hash_strlow(data.as_mut_ptr(), data.as_ptr() as *mut _, data.len()) }
}

foreign_type! {
    pub unsafe type Hash<T>: Send {
        type CType = ffi::ngx_hash_t;
        type PhantomData = T;

        fn drop = never_drop::<ffi::ngx_hash_t>;
    }
}

impl<T> HashRef<T> {
    pub fn find(&self, key: usize, name: &str) -> Option<NonNull<T>> {
        NonNull::new(unsafe {
            ffi::ngx_hash_find(self.as_ptr(), key, name.as_ptr() as *mut _, name.len()).cast()
        })
    }
}

foreign_type! {
    pub unsafe type Elt<T>: Send {
        type CType = ffi::ngx_hash_elt_t;
        type PhantomData = T;

        fn drop = never_drop::<ffi::ngx_hash_elt_t>;
    }
}

impl<T> EltRef<T> {
    pub fn name(&self) -> &str {
        unsafe {
            let r = self.as_raw();

            str::from_utf8_unchecked(slice::from_raw_parts(r.name.as_ptr(), r.len as usize))
        }
    }

    pub fn value(&self) -> Option<NonNull<T>> {
        NonNull::new(unsafe { self.as_raw().value.cast() })
    }
}

foreign_type! {
    pub unsafe type Key<T>: Send {
        type CType = ffi::ngx_hash_key_t;
        type PhantomData = T;

        fn drop = never_drop::<ffi::ngx_hash_key_t>;
    }
}

foreign_type! {
    pub unsafe type Wildcard<T>: Send {
        type CType = ffi::ngx_hash_wildcard_t;
        type PhantomData = T;

        fn drop = never_drop::<ffi::ngx_hash_wildcard_t>;
    }
}

impl<T> WildcardRef<T> {
    pub fn find_head(&self, name: &str) -> Option<NonNull<T>> {
        NonNull::new(unsafe {
            ffi::ngx_hash_find_wc_head(self.as_ptr(), name.as_ptr() as *mut _, name.len()).cast()
        })
    }

    pub fn find_tail(&self, name: &str) -> Option<NonNull<T>> {
        NonNull::new(unsafe {
            ffi::ngx_hash_find_wc_tail(self.as_ptr(), name.as_ptr() as *mut _, name.len()).cast()
        })
    }
}

foreign_type! {
    pub unsafe type Combined<T>: Send {
        type CType = ffi::ngx_hash_combined_t;
        type PhantomData = T;

        fn drop = never_drop::<ffi::ngx_hash_combined_t>;
    }
}

impl<T> CombinedRef<T> {
    pub fn find(&self, key: usize, name: &str) -> Option<NonNull<T>> {
        NonNull::new(unsafe {
            ffi::ngx_hash_find_combined(self.as_ptr(), key, name.as_ptr() as *mut _, name.len())
                .cast()
        })
    }
}

foreign_type! {
    pub unsafe type TableElt: Send {
        type CType = ffi::ngx_table_elt_t;

        fn drop = never_drop::<ffi::ngx_table_elt_t>;
    }
}

impl TableEltRef {
    pub fn hash(&self) -> usize {
        unsafe { self.as_raw().hash }
    }

    pub fn key(&self) -> Option<Str> {
        unsafe { Str::from_raw(self.as_raw().key) }
    }

    pub fn value(&self) -> Option<Str> {
        unsafe { Str::from_raw(self.as_raw().value) }
    }

    pub fn lowcase_key(&self) -> Option<&str> {
        unsafe {
            let r = self.as_raw();

            NonNull::new(r.lowcase_key)
                .map(|p| str::from_utf8_unchecked(slice::from_raw_parts(p.as_ptr(), r.key.len)))
        }
    }

    pub fn next(&self) -> Option<&Self> {
        unsafe { Self::from_raw(self.as_raw().next.cast()) }
    }
}
