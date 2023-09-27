use std::ptr::NonNull;
use std::slice;

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::{fake_drop, ffi, AsRaw};

use super::PoolRef;

foreign_type! {
    pub unsafe type Buf: Send {
        type CType = ffi::ngx_buf_t;

        fn drop = fake_drop::<ffi::ngx_buf_t>;
    }
}

impl BufRef {
    pub fn in_memory(&self) -> bool {
        let r = unsafe { self.as_raw() };

        r.temporary() != 0 || r.memory() != 0 || r.mmap() != 0
    }

    pub fn in_memory_only(&self) -> bool {
        self.in_memory() && unsafe { self.as_raw() }.in_file() == 0
    }

    /// Returns `true` if the buffer is empty, i.e., it has zero length.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

/// Creates a buffer of the specified size in the memory pool.
pub fn new(p: &PoolRef, len: usize) -> Option<&BufRef> {
    NonNull::new(unsafe { ffi::ngx_create_temp_buf(p.as_ptr(), len) })
        .map(|p| unsafe { BufRef::from_ptr(p.as_ptr()) })
}
