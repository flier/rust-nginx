use std::slice;
use std::{mem, ptr::NonNull};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{ffi, flag, get, never_drop, AsRawMut, AsRawRef};

use super::PoolRef;

foreign_type! {
    pub unsafe type Buf: Send {
        type CType = ffi::ngx_buf_t;

        fn drop = never_drop::<ffi::ngx_buf_t>;
    }
}

impl PoolRef {
    /// Creates a buffer of the specified size in the memory pool.
    pub fn create_temp_buf(&mut self, len: usize) -> Option<&BufRef> {
        NonNull::new(unsafe { ffi::ngx_create_temp_buf(self.as_ptr(), len) })
            .map(|p| unsafe { BufRef::from_ptr(p.as_ptr()) })
    }
}

impl BufRef {
    get!(shadow as &BufRef);

    flag!(temporary());
    flag!(memory());
    flag!(mmap());
    flag!(recycled());
    flag!(in_file());
    flag!(flush());
    flag!(sync());
    flag!(last_buf());
    flag!(last_in_chain());
    flag!(last_shadow());
    flag!(temp_file());
    get!(num: i32);

    pub fn in_memory(&self) -> bool {
        self.temporary() || self.memory() || self.mmap()
    }

    pub fn in_memory_only(&self) -> bool {
        self.in_memory() && !self.in_file()
    }

    pub fn special(&self) -> bool {
        (self.flush() || self.last_buf() || self.sync()) && !self.in_memory() && !self.in_file()
    }

    pub fn sync_only(&self) -> bool {
        (self.sync() && !self.in_memory()) && !self.in_file() && !self.flush() && !self.last_buf()
    }

    /// Returns `true` if the buffer is empty, i.e., it has zero length.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn cap(&self) -> isize {
        unsafe {
            let r = self.as_raw();

            r.end.offset_from(r.start)
        }
    }

    /// Returns the length of the buffer contents.
    pub fn len(&self) -> isize {
        unsafe {
            let r = self.as_raw();

            if self.in_memory() {
                r.last.offset_from(r.pos)
            } else {
                r.file_last.wrapping_sub(r.file_pos) as isize
            }
        }
    }

    /// Returns the buffer contents as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let r = self.as_raw();

            slice::from_raw_parts(r.pos, self.len() as usize)
        }
    }

    /// Returns a mutable reference to the buffer contents as a byte slice.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            let r = self.as_raw();

            slice::from_raw_parts_mut(r.pos, self.len() as usize)
        }
    }
}

foreign_type! {
    pub unsafe type Chain: Send {
        type CType = ffi::ngx_chain_t;

        fn drop = never_drop::<ffi::ngx_chain_t>;
    }
}

impl ChainRef {
    pub fn buf(&self) -> &BufRef {
        unsafe { BufRef::from_ptr(self.as_raw().buf) }
    }

    pub fn next(&self) -> Option<&Self> {
        unsafe { NonNull::new(self.as_raw().next).map(|p| ChainRef::from_ptr(p.as_ptr())) }
    }
}

impl PoolRef {
    pub fn create_chain_of_bufs(&mut self, num: isize, size: usize) -> Option<&ChainRef> {
        let bufs = ffi::ngx_bufs_t { num, size };

        unsafe {
            NonNull::new(ffi::ngx_create_chain_of_bufs(
                self.as_ptr(),
                &bufs as *const _ as *mut _,
            ))
            .map(|p| ChainRef::from_ptr(p.as_ptr()))
        }
    }

    pub fn free_chain(&mut self, chain: &mut ChainRef) {
        unsafe {
            chain.as_raw_mut().next = mem::replace(&mut self.as_raw_mut().chain, chain.as_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Log, Pool};

    use super::*;

    #[test]
    fn buf() {
        let mut p = Pool::new(4096, Log::stderr()).unwrap();
        let b = p.create_temp_buf(64).unwrap();
        assert!(b.is_empty());
        assert_eq!(b.len(), 0);
        assert!(b.temporary());
        assert!(b.in_memory());
        assert!(b.in_memory_only());
    }

    #[test]
    fn chain() {
        let mut p = Pool::new(4096, Log::stderr()).unwrap();

        let c = p.create_chain_of_bufs(2, 64).unwrap();
        assert_eq!(c.buf().cap(), 64);
        assert!(c.next().is_some());

        let c2 = c.next().unwrap();
        assert_eq!(c2.buf().cap(), 64);
        assert!(c2.next().is_none());
    }
}
