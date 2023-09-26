use std::ptr::NonNull;
use std::slice;

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::ffi;

use super::PoolRef;

foreign_type! {
    pub unsafe type Buf: Send {
        type CType = ffi::ngx_buf_t;

        fn drop = fake_drop;
    }
}

fn fake_drop(_: *mut ffi::ngx_buf_t) {
    unreachable!()
}

impl BufRef {
    pub fn in_memory(&self) -> bool {
        let r = self.as_raw();

        r.temporary() != 0 || r.memory() != 0 || r.mmap() != 0
    }

    pub fn in_memory_only(&self) -> bool {
        self.in_memory() && self.as_raw().in_file() == 0
    }

    /// Returns `true` if the buffer is empty, i.e., it has zero length.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the length of the buffer contents.
    pub fn len(&self) -> isize {
        let r = self.as_raw();

        unsafe {
            if self.in_memory() {
                r.last.offset_from(r.pos)
            } else {
                r.file_last.wrapping_sub(r.file_pos) as isize
            }
        }
    }

    /// Returns the buffer contents as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        let r = self.as_raw();

        unsafe { slice::from_raw_parts(r.pos, self.len() as usize) }
    }

    /// Returns a mutable reference to the buffer contents as a byte slice.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        let r = self.as_raw();

        unsafe { slice::from_raw_parts_mut(r.pos, self.len() as usize) }
    }

    fn as_raw(&self) -> &ffi::ngx_buf_t {
        unsafe { self.as_ptr().as_ref().expect("buf") }
    }
}

impl PoolRef {
    /// Creates a buffer of the specified size in the memory pool.
    pub fn create_buffer(&self, len: usize) -> Option<&BufRef> {
        NonNull::new(unsafe { ffi::ngx_create_temp_buf(self.as_ptr(), len) })
            .map(|p| unsafe { BufRef::from_ptr(p.as_ptr()) })
    }
}
