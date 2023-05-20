use std::{
    io,
    mem::MaybeUninit,
    ptr,
    slice,
};

use tokio::io::ReadBuf;

use crate::{
    raw::FlowPacketFlags,
    traits::ComposeRead,
};

pub const MIN_READ_BUFFER: usize = 64;

pub struct FlowReader<R> {
    buffer: Vec<u8>,
    cursor: usize,
    reader: R,
}

impl<R: ComposeRead> FlowReader<R> {
    pub async fn read_forward_header(
        &mut self,
    ) -> io::Result<(FlowPacketFlags, u16)> {
        if self.free() < 3 {
            self.copy_to_start();
        }

        while self.unhandled_bytes() < 3 {
            self.read_chunk().await?;
        }

        let (flags, length) = {
            let buffer = self.buffer();
            let last = buffer[2] as u16;

            (buffer[0], (last << 8) | (buffer[1] as u16))
        };

        unsafe { self.consume(3) };

        Ok((FlowPacketFlags::from_bits_truncate(flags), length))
    }

    pub fn consume_max(&mut self, max: usize) -> &[u8] {
        let amt = self.unhandled_bytes().min(max);
        let prev_cursor = self.cursor;

        unsafe {
            self.consume(amt);

            let slice = self
                .buffer
                .get_unchecked(prev_cursor..prev_cursor + amt);
            slice
        }
    }
}

impl<R: ComposeRead> FlowReader<R> {
    pub fn buffer(&self) -> &[u8] {
        debug_assert!(self.cursor <= self.buffer.len());
        unsafe { self.buffer.get_unchecked(self.cursor..) }
    }

    pub unsafe fn consume(&mut self, nbytes: usize) {
        debug_assert!(self.cursor + nbytes <= self.buffer.len());
        self.cursor += nbytes;
    }

    pub fn copy_to_start(&mut self) {
        if self.cursor == 0 {
            return;
        }

        let need_to_copy = self.unhandled_bytes();
        unsafe {
            ptr::copy(
                self.buffer.as_ptr().add(self.cursor),
                self.buffer.as_mut_ptr(),
                need_to_copy,
            );

            self.cursor = 0;
            self.buffer.set_len(need_to_copy);
        }
    }

    async fn read_chunk(&mut self) -> io::Result<usize> {
        let len = self.buffer.len();
        let read = self
            .reader
            .read_buf(&mut ReadBuf::uninit(self.buffer.spare_capacity_mut()))
            .await?;
        if read == 0 {
            return Err(io::Error::last_os_error());
        }

        unsafe { self.buffer.set_len(len + read) };
        Ok(read)
    }

    /// # Panics
    ///
    /// panics if buffer size is less than
    /// [`MIN_READ_BUFFER`]
    pub fn with_capacity(reader: R, capacity: usize) -> Self {
        assert!(capacity >= MIN_READ_BUFFER);

        Self {
            buffer: Vec::with_capacity(capacity),
            cursor: 0,
            reader,
        }
    }

    pub fn take_slice_and_raw<'a>(
        &'a mut self,
        length: usize,
    ) -> (
        Result<&'a mut [MaybeUninit<u8>], &'a [u8]>,
        usize,
        &'a mut R,
    ) {
        let ptr = self.buffer.as_mut_ptr();
        let capacity = self.buffer.capacity();
        let initialized = self.unhandled_bytes().min(length);

        (
            if capacity
                < self
                    .buffer
                    .len()
                    .wrapping_add(length - initialized)
            {
                Err(unsafe {
                    slice::from_raw_parts(ptr.add(self.cursor), initialized)
                })
            } else {
                Ok(unsafe {
                    slice::from_raw_parts_mut(
                        ptr.add(self.cursor).cast(),
                        length,
                    )
                })
            },
            initialized,
            &mut self.reader,
        )
    }

    pub fn spare_and_raw<'a>(
        &'a mut self,
    ) -> (&'a mut [MaybeUninit<u8>], &'a mut R) {
        (self.buffer.spare_capacity_mut(), &mut self.reader)
    }

    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.buffer.spare_capacity_mut()
    }

    #[inline]
    fn unhandled_bytes(&self) -> usize {
        self.buffer.len() - self.cursor
    }

    #[inline]
    pub fn free(&self) -> usize {
        self.buffer.capacity() - self.buffer.len()
    }
}
