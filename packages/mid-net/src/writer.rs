use std::{
    future::Future,
    io::{
        self,
        IoSlice,
    },
};

use mid_compression::interface::ICompressor;
use tokio::io::{
    AsyncWrite,
    AsyncWriteExt,
    BufWriter,
};

use crate::{
    compression::{
        CompressionAlgorithm,
        CompressionStatus,
        ForwardCompression,
    },
    proto::PacketType,
    utils::{
        self,
        encode_fwd_header,
        ident_type,
        FancyUtilExt,
    },
};

pub struct MidClientWriter<'a, W, C> {
    inner: &'a mut MidWriter<W, C>,
}

pub struct MidServerWriter<'a, W, C> {
    inner: &'a mut MidWriter<W, C>,
}

/// Write side of the `Middleware` protocol
pub struct MidWriter<W, C> {
    inner: W,
    compressor: C,
}

impl<'a, W, C> MidServerWriter<'a, W, C>
where
    W: AsyncWriteExt + Unpin,
{
    /// Write ping response to the client.
    pub async fn write_ping(
        &mut self,
        server_name: &str,
        algorithm: CompressionAlgorithm,
        buffer_size: u16,
    ) -> io::Result<()> {
        self.inner
            .write_two_bufs(
                &[
                    ident_type(PacketType::Ping as u8),
                    algorithm as u8,
                    (buffer_size & 0xff) as u8,
                    (buffer_size >> 8) as u8,
                    server_name
                        .len()
                        .try_into()
                        .expect("`server_name` is greater than `u8::MAX`"),
                ],
                server_name.as_bytes(),
            )
            .await
            .unitize_io()
    }
}

// Common writer methods

impl<W, C> MidWriter<W, C>
where
    W: AsyncWriteExt + Unpin,
    C: ICompressor,
{
    async fn write_forward_impl(
        &mut self,
        client_id: u16,
        buffer: &[u8],
        compressed: bool,
    ) -> io::Result<()> {
        let (header, header_size) = encode_fwd_header(
            client_id,
            buffer
                .len()
                .try_into()
                .expect("Buffer size exceeds `u16::MAX`"),
            compressed,
        );
        self.write_two_bufs(&header[..header_size], buffer)
            .await
            .unitize_io()
    }

    pub async fn write_forward(
        &mut self,
        client_id: u16,
        buffer: &[u8],
        compression: ForwardCompression,
    ) -> io::Result<CompressionStatus> {
        fn uncompressed(
            in_: io::Result<()>,
        ) -> io::Result<CompressionStatus> {
            in_.map(|()| CompressionStatus::Uncompressed)
        }

        match compression {
            ForwardCompression::Compress { with_threshold }
                if with_threshold <= buffer.len() =>
            {
                let mut preallocated = Vec::with_capacity(buffer.len());
                if let Ok(compressed) = self
                    .compressor
                    .try_compress(buffer, &mut preallocated)
                {
                    if compressed.get() > buffer.len() {
                        // Yeah, this is possible
                        uncompressed(
                            self.write_forward_impl(
                                client_id, buffer, false,
                            )
                            .await,
                        )
                    } else {
                        let status = CompressionStatus::Compressed {
                            before: buffer.len(),
                            after: compressed.get(),
                        };

                        self.write_forward_impl(client_id, buffer, true)
                            .await
                            .map(move |()| status)
                    }
                } else {
                    uncompressed(
                        self.write_forward_impl(client_id, buffer, false)
                            .await,
                    )
                }
            }
            _ => uncompressed(
                self.write_forward_impl(client_id, buffer, false)
                    .await,
            ),
        }
    }
}

impl<W, C> MidWriter<W, C>
where
    W: AsyncWriteExt + Unpin,
{
    /// Write two buffers to the socket in vectored mode.
    ///
    /// Returns
    /// - Ok(true) if buffer was wrote using efficient
    ///   implementation (without allocating buffer with
    ///   size before.len() + after.len())
    /// - Ok(false) if buffer was wrote using the fallback
    ///   way (allocating buffer with size before.len() +
    ///   after.len() and copying data to it)
    pub async fn write_two_bufs(
        &mut self,
        before: &[u8],
        after: &[u8],
    ) -> io::Result<bool> {
        let (blen, alen) = (before.len(), after.len());
        let total = blen + alen;

        if !self.inner.is_write_vectored() {
            let mut buf = Vec::with_capacity(total);

            // SAFETY: this is safe since `Vec::with_capacity` will
            // return buffer with at least `total` capacity and its data
            // will be initialized.
            // Possibly it can be done better? Without buffer
            // pre-filling
            unsafe {
                std::ptr::copy_nonoverlapping(
                    before.as_ptr(),
                    buf.as_mut_ptr(),
                    before.len(),
                );

                std::ptr::copy_nonoverlapping(
                    after.as_ptr(),
                    buf.as_mut_ptr().offset(
                        before.len().try_into().expect(
                            "Failed to copy to a single buffer: too long \
                             `before` buffer size",
                        ),
                    ),
                    after.len(),
                );

                buf.set_len(total);
            };

            self.inner.write_all(&buf).await?;
            return Ok(false);
        }

        let mut written: usize = 0;
        let mut ios = [IoSlice::new(before), IoSlice::new(after)];

        loop {
            let wrote = self.inner.write_vectored(&ios).await?;
            written += wrote;

            if written < total {
                if written >= blen {
                    break self
                        .inner
                        .write_all(&after[(written - blen)..])
                        .await
                        .map(|_| true);
                }

                ios[0] = IoSlice::new(&before[written..]);
            } else {
                break Ok(true);
            }
        }
    }

    /// Writes entire buffer into the socket
    pub fn write_all<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> impl Future<Output = io::Result<()>> + 'a {
        self.inner.write_all(buf)
    }

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

    /// Create client packets writer. Used mainly to
    /// incapsulate client and server packets
    pub fn client(&mut self) -> MidClientWriter<'_, W, C> {
        MidClientWriter { inner: self }
    }

    /// Same as [`MidWriter::client`] but for server packets
    pub fn server(&mut self) -> MidServerWriter<'_, W, C> {
        MidServerWriter { inner: self }
    }
}

// Bufferization & creation related stuff

impl<W, C> MidWriter<BufWriter<W>, C>
where
    W: AsyncWrite + Unpin,
{
    /// Flush underlying write buffer, so remote side will
    /// receive buffered bytes immediately
    pub fn flush(&mut self) -> impl Future<Output = io::Result<()>> + '_ {
        self.inner.flush()
    }
}

impl<W, C> MidWriter<BufWriter<W>, C>
where
    W: AsyncWrite,
{
    /// Create buffered writer.
    pub fn new_buffered(
        socket: W,
        compressor: C,
        buffer_size: usize,
    ) -> Self {
        Self {
            inner: BufWriter::with_capacity(buffer_size, socket),
            compressor,
        }
    }

    /// Remove bufferization from the writer.
    ///
    /// WARNING: it is neccessary to call
    /// [`MidWriter::flush`] before the unbuffering so
    /// you're sure that previously buffered data was wrote
    pub fn unbuffer(self) -> MidWriter<W, C> {
        MidWriter {
            inner: self.inner.into_inner(),
            compressor: self.compressor,
        }
    }
}

impl<W, C> MidWriter<W, C>
where
    W: AsyncWrite,
{
    /// Make writer buffered
    pub fn make_buffered(
        self,
        buffer_size: usize,
    ) -> MidWriter<BufWriter<W>, C> {
        MidWriter::new_buffered(self.inner, self.compressor, buffer_size)
    }
}

impl<W, C> MidWriter<W, C> {
    /// Get shared access to the underlying socket.
    pub const fn socket(&self) -> &W {
        &self.inner
    }

    /// Get exclusive access to the underlying socket.
    pub fn socket_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    /// Simply create writer from the underlying socket
    pub const fn new(socket: W, compressor: C) -> Self {
        Self {
            inner: socket,
            compressor,
        }
    }
}
