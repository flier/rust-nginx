use std::mem::MaybeUninit;
use std::ptr::{self, NonNull};
use std::{ffi::c_void, mem};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{core::LogRef, ffi, native_callback, never_drop, AsRawRef, Error};

foreign_type! {
    pub unsafe type Pool: Send {
        type CType = ffi::ngx_pool_t;

        fn drop = ffi::ngx_destroy_pool;
    }
}

impl Pool {
    pub fn new(size: usize, log: &LogRef) -> Result<Self, Error> {
        NonNull::new(unsafe { ffi::ngx_create_pool(size, log.as_ptr()) })
            .map(Pool)
            .ok_or(Error::OutOfMemory)
    }
}

impl PoolRef {
    #[inline(always)]
    pub fn reset(&self) {
        unsafe {
            ffi::ngx_reset_pool(self.as_ptr());
        }
    }

    /// Allocates aligned memory from the pool of the specified size.
    ///
    /// Returns a raw pointer to the allocated memory.
    ///
    /// # Safety
    ///
    /// This function is marked as unsafe due to the raw pointer manipulation.
    #[inline(always)]
    pub unsafe fn palloc(&self, size: usize) -> *mut c_void {
        ffi::ngx_palloc(self.as_ptr(), size)
    }

    /// Allocate unaligned memory from the specified pool.
    ///
    /// Returns a raw pointer to the allocated memory.
    ///
    /// Mostly used for allocating strings.
    ///
    /// # Safety
    ///
    /// This function is marked as unsafe due to the raw pointer manipulation.
    #[inline(always)]
    pub unsafe fn pnalloc(&self, size: usize) -> *mut c_void {
        ffi::ngx_pnalloc(self.as_ptr(), size)
    }

    /// Allocate aligned memory from the specified pool and fill it with zeroes.
    ///
    /// Returns a raw pointer to the allocated memory.
    ///
    /// # Safety
    ///
    /// This function is marked as unsafe due to the raw pointer manipulation.
    #[inline(always)]
    pub unsafe fn pcalloc(&self, size: usize) -> *mut c_void {
        ffi::ngx_pcalloc(self.as_ptr(), size)
    }

    /// Allocates aligned large memory from the pool.
    ///
    /// Returns a raw pointer to the allocated memory.
    ///
    /// # Safety
    ///
    /// This function is marked as unsafe due to the raw pointer manipulation.
    #[inline(always)]
    pub unsafe fn pmemalign(&self, size: usize, alignment: usize) -> *mut c_void {
        ffi::ngx_pmemalign(self.as_ptr(), size, alignment)
    }

    /// Free large memory from the pool.
    ///
    /// Returns `true` if successful, or `false` if memory is not allocated from the pool.
    ///
    /// # Safety
    ///
    /// This function is marked as unsafe due to the raw pointer manipulation.
    #[inline(always)]
    pub unsafe fn pfree<T>(&self, p: *mut T) -> bool {
        ffi::ngx_pfree(self.as_ptr(), p.cast()) == ffi::NGX_OK as isize
    }

    /// Allocates memory for a type from the pool.
    ///
    /// Returns a typed pointer to the allocated memory.
    pub fn alloc<T>(&self) -> Option<&mut MaybeUninit<T>> {
        unsafe { NonNull::new(self.palloc(mem::size_of::<T>())).map(|p| p.cast().as_mut()) }
    }

    /// Allocates zeroed memory for a type from the pool.
    ///
    /// Returns a typed pointer to the allocated memory.
    pub fn calloc<T>(&self) -> Option<&mut T> {
        unsafe { NonNull::new(self.pcalloc(mem::size_of::<T>())).map(|p| p.cast().as_mut()) }
    }

    /// Allocates memory for a value of a specified type and adds a cleanup handler to the memory pool.
    ///
    /// Returns a typed reference to the allocated memory if successful,
    /// or `None` if allocation or cleanup handler addition fails.
    pub fn allocate_default<T>(&self) -> Option<&mut T>
    where
        T: Default,
    {
        self.allocate::<T>(Default::default())
    }

    /// Allocates memory for a value of a specified type and adds a cleanup handler to the memory pool.
    ///
    /// Returns a typed reference to the allocated memory if successful,
    /// or `None` if allocation or cleanup handler addition fails.
    pub fn allocate<T>(&self, value: T) -> Option<&mut T> {
        unsafe {
            NonNull::new(self.palloc(mem::size_of::<T>()).cast()).and_then(|mut p| {
                ptr::write(p.as_ptr(), value);

                if self.add_cleanup(Some(cleanup_type::<T>), Some(p)).is_ok() {
                    Some(p.as_mut())
                } else {
                    ptr::drop_in_place(p.as_ptr());

                    None
                }
            })
        }
    }

    /// Adds a cleanup handler to the memory pool.
    ///
    /// If the `data` contains a value, it will be passed to the cleanup `handler`;
    /// if the `data` is `None` but size of `T` is not zero, some pooled memory will be allocated,
    /// and pass as `data` to the cleanup `handler`;
    /// otherwise, a null will passed as `data` to the cleanup `handler`.
    ///
    /// # Safety
    ///
    /// This function is marked as unsafe due to the raw pointer manipulation.
    /// The caller must ensure that the `data` is valid for the lifetime of the memory pool.
    pub unsafe fn add_cleanup<T>(
        &self,
        handler: ffi::ngx_pool_cleanup_pt,
        data: Option<NonNull<T>>,
    ) -> Result<Option<&mut MaybeUninit<T>>, Error> {
        let data = if let Some(p) = data {
            Some(p.cast().as_mut())
        } else if mem::size_of::<T>() > 0 {
            Some(self.alloc::<T>().ok_or(Error::OutOfMemory)?)
        } else {
            None
        };

        ffi::ngx_pool_cleanup_add(self.as_ptr(), 0)
            .as_mut()
            .map(|p| {
                p.handler = handler;
                p.data = data.map_or_else(ptr::null_mut, |p| p.as_mut_ptr() as *mut _);
                p.data.cast::<MaybeUninit<T>>().as_mut()
            })
            .ok_or(Error::OutOfMemory)
    }

    pub fn cleanups(&self) -> Cleanups {
        Cleanups(unsafe {
            NonNull::new(self.as_raw().cleanup).map(|p| CleanupRef::from_ptr(p.as_ptr()))
        })
    }
}

/// Cleanup handler for a specific type `T`.
///
/// This function is called when cleaning up a value of type `T` in an FFI context.
///
/// # Safety
///
/// This function is marked as unsafe due to the raw pointer manipulation and the assumption that `data` is a valid pointer to `T`.
///
/// # Arguments
///
/// * `data` - A raw pointer to the value of type `T` to be cleaned up.
unsafe extern "C" fn cleanup_type<T>(data: *mut c_void) {
    if data.is_null() {
        ptr::drop_in_place(data.cast::<T>());
    }
}

pub struct Cleanups<'a>(Option<&'a CleanupRef>);

impl<'a> Iterator for Cleanups<'a> {
    type Item = &'a CleanupRef;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(p) = self.0.take() {
            self.0 = p.next();
            Some(p)
        } else {
            None
        }
    }
}

foreign_type! {
    pub unsafe type Cleanup: Send {
        type CType = ffi::ngx_pool_cleanup_t;

        fn drop = never_drop::<ffi::ngx_pool_cleanup_t>;
    }
}

impl CleanupRef {
    callback! {
        handler: CleanupFn;
    }

    property! {
        next as &CleanupRef;
    }

    pub fn raw_handler(&self) -> ffi::ngx_pool_cleanup_pt {
        unsafe { self.as_raw().handler }
    }

    pub fn data<T>(&self) -> Option<&mut T> {
        unsafe { self.as_raw().data.cast::<T>().as_mut() }
    }
}

#[native_callback]
pub type CleanupFn<T> = fn(data: Option<&T>);

#[cfg(test)]
mod tests {
    use crate::core::Log;

    use super::*;

    #[test]
    fn pool() {
        let p = Pool::new(4096, Log::stderr()).unwrap();

        let v = p.calloc::<usize>();
        assert!(v.is_some());
        assert!(!unsafe { p.pfree(v.unwrap()) });

        let v1: Option<&mut MaybeUninit<[u8; 4096]>> = p.alloc::<[u8; 4096]>();
        assert!(v1.is_some());
        assert!(unsafe { p.pfree(v1.unwrap().as_mut_ptr()) });
    }
}
