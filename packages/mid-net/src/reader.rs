use std::{
    future::{
        poll_fn,
        Future,
    },
    io,
    pin::Pin,
};

use tokio::io::{
    AsyncRead,
    AsyncReadExt,
    BufReader,
    ReadBuf,
};

pub struct MidReader<R> {
    inner: R,
}

// Actual reading

impl<R> MidReader<R>
where
    R: AsyncReadExt + Unpin,
{
    /// Read `u8` from the underlying stream
    pub fn read_u8(
        &mut self,
    ) -> impl Future<Output = io::Result<u8>> + '_ {
        self.inner.read_u8()
    }

    /// Read `u16` from the underlying stream (little
    /// endian)
    pub fn read_u16(
        &mut self,
    ) -> impl Future<Output = io::Result<u16>> + '_ {
        self.inner.read_u16_le()
    }

    /// Read `u32` from the underlying stream (little
    /// endian)
    pub fn read_u32(
        &mut self,
    ) -> impl Future<Output = io::Result<u32>> + '_ {
        self.inner.read_u32_le()
    }

    /// Read `size` bytes from the socket without buffer
    /// pre-filling.
    pub async fn read_buffer(
        &mut self,
        size: usize,
    ) -> io::Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::with_capacity(size);
        {
            let mut read_buf =
                ReadBuf::uninit(&mut buffer.spare_capacity_mut()[..size]);

            while read_buf.filled().len() < size {
                poll_fn(|cx| {
                    Pin::new(&mut self.inner).poll_read(cx, &mut read_buf)
                })
                .await?;
            }
        }

        // SAFETY: this is safe since we passed
        // `read_buf.filled().len() >= size` condition,
        // so `buffer` initialized with exactly `size` items.
        unsafe { buffer.set_len(size) }
        Ok(buffer)
    }
}

// Bufferization & creation related stuff

impl<R> MidReader<R>
where
    R: AsyncRead,
{
    /// Create buffered reader (wraps R with `BufReader<R>`
    /// with specified capacity)
    pub fn make_buffered(
        self,
        buffer_size: usize,
    ) -> MidReader<BufReader<R>> {
        MidReader::new_buffered(self.inner, buffer_size)
    }
}

impl<R> MidReader<BufReader<R>>
where
    R: AsyncRead,
{
    /// Create buffered version of the reader
    pub fn new_buffered(socket: R, buffer_size: usize) -> Self {
        Self {
            inner: BufReader::with_capacity(buffer_size, socket),
        }
    }

    /// Remove underlying buffer.
    ///
    /// WARNING: buffered data can be lost!
    pub fn unbuffer(self) -> MidReader<R> {
        MidReader {
            inner: self.inner.into_inner(),
        }
    }
}

impl<R> MidReader<R> {
    /// Get shared access to the underlying socket
    pub const fn socket(&self) -> &R {
        &self.inner
    }

    /// Get exclusive access to the underlying socket
    pub fn socket_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Simply create reader from the underlying socket type
    pub const fn new(socket: R) -> Self {
        Self { inner: socket }
    }
}
