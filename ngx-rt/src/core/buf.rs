use std::slice;
use std::{mem, ptr::NonNull};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{ffi, flag, never_drop, property, AsRawMut, AsRawRef};

use super::{FileRef, PoolRef};

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
    property! {
        /// File object.
        file as &FileRef;

        /// Buffer shadow.
        shadow as &BufRef;
    }

    flag! {
        /// the buffer references writable memory.
        temporary();

        /// the buffer references read-only memory.
        memory();

        /// the buffer references data in a mmapped file.
        mmap();

        /// the buffer can be reused and needs to be consumed as soon as possible.
        recycled();

        /// the buffer references data in a file.
        in_file();

        /// all data prior to the buffer need to be flushed.
        flush();

        /// the buffer carries no data or special signal like flush or last_buf.
        sync();

        /// the buffer is the last in output.
        last_buf();

        /// there are no more data buffers in a request or subrequest.
        last_in_chain();

        /// the buffer is the last one that references a particular shadow buffer.
        last_shadow();

        /// the buffer is in a temporary file.
        temp_file();
    }

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

    pub fn cap(&self) -> usize {
        unsafe {
            let r = self.as_raw();

            assert!(r.end >= r.start);

            r.end.offset_from(r.start) as usize
        }
    }

    /// Returns the length of the buffer contents.
    pub fn len(&self) -> usize {
        unsafe {
            let r = self.as_raw();

            if self.in_memory() {
                assert!(r.last >= r.pos);

                r.last.offset_from(r.pos) as usize
            } else {
                assert!(r.file_last >= r.file_pos);

                r.file_last.wrapping_sub(r.file_pos) as usize
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
    pub fn alloc_chain_link(&mut self) -> Option<&mut ChainRef> {
        unsafe {
            NonNull::new(ffi::ngx_alloc_chain_link(self.as_ptr()))
                .map(|p| ChainRef::from_ptr_mut(p.as_ptr()))
        }
    }

    pub fn create_chain_of_bufs(&mut self, num: isize, size: usize) -> Option<&mut ChainRef> {
        let bufs = ffi::ngx_bufs_t { num, size };

        unsafe {
            NonNull::new(ffi::ngx_create_chain_of_bufs(
                self.as_ptr(),
                &bufs as *const _ as *mut _,
            ))
            .map(|p| ChainRef::from_ptr_mut(p.as_ptr()))
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
