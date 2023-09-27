use std::ptr::{self, NonNull};
use std::{ffi::c_void, mem};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{ffi, Error};

use super::LogRef;

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
    pub fn reset(&self) {
        unsafe {
            ffi::ngx_reset_pool(self.as_ptr());
        }
    }

    /// Allocates memory from the pool of the specified size.
    ///
    /// Returns a raw pointer to the allocated memory.
    pub unsafe fn palloc(&self, size: usize) -> *mut c_void {
        ffi::ngx_palloc(self.as_ptr(), size)
    }

    /// Allocates aligned memory from the pool of the specified size.
    ///
    /// Returns a raw pointer to the allocated memory.
    pub unsafe fn pnalloc(&self, size: usize) -> *mut c_void {
        ffi::ngx_pnalloc(self.as_ptr(), size)
    }

    /// Allocates zeroed memory from the pool of the specified size.
    ///
    /// Returns a raw pointer to the allocated memory.
    pub unsafe fn pcalloc(&self, size: usize) -> *mut c_void {
        ffi::ngx_pcalloc(self.as_ptr(), size)
    }

    /// Allocates aligned large memory from the pool.
    ///
    /// Returns a raw pointer to the allocated memory.
    pub unsafe fn pmemalign(&self, size: usize, alignment: usize) -> *mut c_void {
        ffi::ngx_pmemalign(self.as_ptr(), size, alignment)
    }

    /// Free large memory from the pool.
    ///
    /// Returns `true` if successful, or `false` if memory is not allocated from the pool.
    pub unsafe fn pfree<T>(&self, p: NonNull<T>) -> bool {
        ffi::ngx_pfree(self.as_ptr(), p.as_ptr().cast()) == ffi::NGX_OK as isize
    }

    /// Allocates memory for a type from the pool.
    ///
    /// Returns a typed pointer to the allocated memory.
    pub fn alloc<T: Copy>(&self) -> Option<NonNull<T>> {
        NonNull::new(unsafe { self.palloc(mem::size_of::<T>()).cast() })
    }

    /// Allocates zeroed memory for a type from the pool.
    ///
    /// Returns a typed pointer to the allocated memory.
    pub fn calloc<T: Copy>(&self) -> Option<NonNull<T>> {
        NonNull::new(unsafe { self.pcalloc(mem::size_of::<T>()).cast() })
    }

    /// Allocates memory for a value of a specified type and adds a cleanup handler to the memory pool.
    ///
    /// Returns a typed pointer to the allocated memory if successful, or a null pointer if allocation or cleanup handler addition fails.
    ///
    /// # Safety
    /// This function is marked as unsafe because it involves raw pointer manipulation.
    pub fn allocate<T>(&self, value: T) -> Option<NonNull<T>> {
        unsafe {
            NonNull::new(self.palloc(mem::size_of::<T>()).cast()).and_then(|p| {
                ptr::write(p.as_ptr(), value);

                if let Ok(_) = self.add_cleanup(p) {
                    Some(p)
                } else {
                    ptr::drop_in_place(p.as_ptr());

                    None
                }
            })
        }
    }

    unsafe fn add_cleanup<T>(&self, value: NonNull<T>) -> Result<(), ()> {
        unsafe { ffi::ngx_pool_cleanup_add(self.as_ptr(), 0) }
            .as_mut()
            .map(|p| {
                p.handler = Some(cleanup_type::<T>);
                p.data = value.as_ptr().cast();
            })
            .ok_or(())
    }
}

/// Cleanup handler for a specific type `T`.
///
/// This function is called when cleaning up a value of type `T` in an FFI context.
///
/// # Safety
/// This function is marked as unsafe due to the raw pointer manipulation and the assumption that `data` is a valid pointer to `T`.
///
/// # Arguments
///
/// * `data` - A raw pointer to the value of type `T` to be cleaned up.
unsafe extern "C" fn cleanup_type<T>(data: *mut c_void) {
    ptr::drop_in_place(data.cast::<T>());
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;

    use crate::core::Log;

    use super::*;

    #[test]
    fn pool() {
        let tmp_dir = temp_dir();
        let log = Log::init(Some(&tmp_dir), Some("")).unwrap();
        let p = Pool::new(4096, log).unwrap();

        let v = p.calloc::<usize>();
        assert!(v.is_some());
        assert!(!unsafe { p.pfree(v.unwrap()) });

        let v1 = p.alloc::<[u8; 4096]>();
        assert!(v1.is_some());
        assert!(unsafe { p.pfree(v1.unwrap()) });
    }
}
