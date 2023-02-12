use std::{
    future::Future,
    io,
};

use tokio::io::{
    AsyncWrite,
    AsyncWriteExt,
    BufWriter,
};

/// Write side of the `Middleware` protocol
pub struct MidWriter<W> {
    inner: W,
}

impl<W> MidWriter<W> where W: AsyncWriteExt + Unpin {}

// Bufferization & creation related stuff

impl<W> MidWriter<BufWriter<W>>
where
    W: AsyncWrite + Unpin,
{
    /// Same as [`MidWriter::write_u32`] but writes u32
    /// (little endian)
    pub fn write_u32(
        &mut self,
        v: u32,
    ) -> impl Future<Output = io::Result<()>> + '_ {
        self.inner.write_u32_le(v)
    }

    /// Same as [`MidWriter::write_u8`] but writes u16
    /// (little endian)
    pub fn write_u16(
        &mut self,
        v: u16,
    ) -> impl Future<Output = io::Result<()>> + '_ {
        self.inner.write_u16_le(v)
    }

    /// Write u8 to the destination socket (or possibly to
    /// buffer)
    pub fn write_u8(
        &mut self,
        v: u8,
    ) -> impl Future<Output = io::Result<()>> + '_ {
        self.inner.write_u8(v)
    }

    /// Flush underlying write buffer, so remote side will
    /// receive buffered bytes immediately
    pub fn flush(&mut self) -> impl Future<Output = io::Result<()>> + '_ {
        self.inner.flush()
    }
}

impl<W> MidWriter<BufWriter<W>>
where
    W: AsyncWrite,
{
    /// Create buffered writer.
    pub fn new_buffered(socket: W, buffer_size: usize) -> Self {
        Self {
            inner: BufWriter::with_capacity(buffer_size, socket),
        }
    }

    /// Remove bufferization from the writer.
    ///
    /// WARNING: it is neccessary to call
    /// [`MidWriter::flush`] before the unbuffering so
    /// you're sure that previously buffered data was wrote
    pub fn unbuffer(self) -> MidWriter<W> {
        MidWriter {
            inner: self.inner.into_inner(),
        }
    }
}

impl<W> MidWriter<W>
where
    W: AsyncWrite,
{
    /// Make writer buffered
    pub fn make_buffered(
        self,
        buffer_size: usize,
    ) -> MidWriter<BufWriter<W>> {
        MidWriter::new_buffered(self.inner, buffer_size)
    }
}

impl<W> MidWriter<W> {
    /// Get shared access to the underlying socket.
    pub const fn socket(&self) -> &W {
        &self.inner
    }

    /// Get exclusive access to the underlying socket.
    pub fn socket_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    /// Simply create writer from the underlying socket
    pub const fn new(socket: W) -> Self {
        Self { inner: socket }
    }
}
