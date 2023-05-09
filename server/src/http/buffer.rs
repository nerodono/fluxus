use std::{
    mem::MaybeUninit,
    ops::Range,
    ptr,
};

#[derive(Debug)]
pub struct BodyBytes {
    pub buffered: usize,
    pub unbuffered: usize,
}

#[derive(Debug)]
pub struct RequestBuffer {
    pub buffer: Vec<u8>,
    pub cursor: usize,
    pub continuation: usize,
}

impl RequestBuffer {
    pub fn move_contents(&mut self) {
        let copy_len = self.buffer.len() - self.cursor;
        unsafe {
            ptr::copy(
                self.buffer.as_ptr().add(self.cursor),
                self.buffer.as_mut_ptr(),
                copy_len,
            );
            self.buffer.set_len(copy_len);
        };
        self.cursor = 0;
    }
}

impl RequestBuffer {
    pub fn as_ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buffer.as_mut_ptr()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn cursor_slice(&self) -> &[u8] {
        &self.buffer[self.cursor..]
    }

    #[inline]
    pub fn calc_recv_sizes(&mut self, size: usize) -> BodyBytes {
        let unhandled = self.cursor_space();
        let buffered = unhandled.min(size);
        BodyBytes {
            buffered,
            unbuffered: size - buffered,
        }
    }

    pub fn search_slice(&self) -> (&[u8], usize) {
        let offset = self.cursor + self.continuation;
        (&self.buffer[offset..], offset)
    }

    #[track_caller]
    pub fn take_range(&self, range: Range<usize>) -> &[u8] {
        &self.buffer[range]
    }

    pub unsafe fn add_size(&mut self, no: usize) {
        self.buffer.set_len(self.buffer.len() + no);
    }

    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.buffer.spare_capacity_mut()
    }

    pub fn cursor_space(&self) -> usize {
        self.buffer.len() - self.cursor
    }

    pub fn free_space(&self) -> usize {
        self.buffer.capacity() - self.buffer.len()
    }

    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(buffer_size),
            cursor: 0,
            continuation: 0,
        }
    }
}
