use std::{
    future::{
        poll_fn,
        Future,
    },
    io,
    pin::Pin,
};

use mid_compression::{
    error::SizeRetrievalError,
    interface::IDecompressor,
};
use tokio::io::{
    AsyncRead,
    AsyncReadExt,
    BufReader,
    ReadBuf,
};

use crate::{
    compression::{
        DecompressionConstraint,
        DecompressionStrategy,
    },
    error,
    utils::{
        self,
        flags,
    },
};

pub struct MidReader<R, D> {
    inner: R,
    decompressor: D,
}

// Actual reading

impl<R, D> MidReader<R, D>
where
    R: AsyncReadExt + Unpin,
    D: IDecompressor,
{
    /// Reads compressed pile of bytes from the stream
    pub async fn read_compressed(
        &mut self,
        size: usize,
        strategy: DecompressionStrategy,
    ) -> Result<Vec<u8>, error::CompressedReadError> {
        let buffer = self.read_buffer(size).await?;
        let dec_size = self.decompressor.try_decompressed_size(&buffer);
        if matches!(dec_size, Err(SizeRetrievalError::InvalidData)) {
            return Err(error::CompressedReadError::InvalidData);
        }

        let mut output = Vec::new();

        match strategy {
            DecompressionStrategy::ConstrainedConst { constraint } => {
                match &constraint {
                    ty @ (DecompressionConstraint::Max(m)
                    | DecompressionConstraint::MaxSizeMultiplier(m)) => {
                        let max_size =
                            if matches!(ty, DecompressionConstraint::Max(..)) {
                                *m
                            } else {
                                size * *m
                            };

                        if let Ok(dec_size) = dec_size {
                            if dec_size > max_size {
                                Err(error::CompressedReadError::ConstraintFailed { constraint: ty.clone() })
                            } else {
                                output.reserve(dec_size);
                                self.decompressor
                                    .try_decompress(&buffer, &mut output)
                                    .map_err(|_| {
                                        error::CompressedReadError::InvalidData
                                    })
                                    .map(move |_| output)
                            }
                        } else {
                            output.reserve(size);
                            while output.capacity() < max_size {
                                if self
                                    .decompressor
                                    .try_decompress(&buffer, &mut output)
                                    .is_ok()
                                {
                                    return Ok(output);
                                }

                                output.reserve(output.capacity());
                            }

                            Err(error::CompressedReadError::ConstraintFailed {
                                constraint: ty.clone(),
                            })
                        }
                    }
                }
            }

            DecompressionStrategy::Unconstrained => {
                if let Ok(size) = dec_size {
                    output.reserve(size);
                    self.decompressor
                        .try_decompress(&buffer, &mut output)
                        .unwrap_or_else(|_| unreachable!());
                    return Ok(output);
                }

                output.reserve(size << 1);
                loop {
                    if self
                        .decompressor
                        .try_decompress(&buffer, &mut output)
                        .is_ok()
                    {
                        return Ok(buffer);
                    }

                    output.reserve(output.capacity());
                }
            }
        }
    }
}

impl<R, D> MidReader<R, D>
where
    R: AsyncReadExt + Unpin,
{
    /// Skips `nbytes` bytes from the underlying stream.
    pub async fn skip_n_bytes(&mut self, nbytes: usize) -> io::Result<()> {
        const CHUNK_SIZE: usize = 128;
        let mut buf = [0; CHUNK_SIZE];
        let mut read = 0;

        while read < nbytes {
            let remaining = (nbytes - read).min(CHUNK_SIZE);
            let current_read = self.inner.read(&mut buf[..remaining]).await?;

            read += current_read;
        }

        Ok(())
    }

    /// Reads packet type and decodes it returning pair of
    /// `u8`'s
    pub async fn read_raw_packet_type(&mut self) -> io::Result<(u8, u8)> {
        self.read_u8().await.map(utils::decode_type)
    }

    /// Reads string of prefixed size with max size of
    /// `u8::MAX`, uses lossy utf8 decoding.
    pub async fn read_string_prefixed(&mut self) -> io::Result<String> {
        let size = self.read_u8().await?;
        self.read_string(size as usize).await
    }

    /// Reads string of size `bytes_size` with lossy utf8
    /// decoding.
    pub async fn read_string(
        &mut self,
        bytes_size: usize,
    ) -> io::Result<String> {
        self.read_buffer(bytes_size)
            .await
            .map(|buf| String::from_utf8_lossy(&buf).into_owned())
    }

    /// Reads prefixed buffer with max size of `u8::MAX`.
    pub async fn read_bytes_prefixed(&mut self) -> io::Result<Vec<u8>> {
        let size = self.read_u8().await?;
        self.read_buffer(size as usize).await
    }

    /// Read `u8` from the underlying stream
    pub fn read_u8(&mut self) -> impl Future<Output = io::Result<u8>> + '_ {
        self.inner.read_u8()
    }

    /// Read `u16` from the underlying stream (little
    /// endian)
    pub fn read_u16(&mut self) -> impl Future<Output = io::Result<u16>> + '_ {
        self.inner.read_u16_le()
    }

    /// Read `u32` from the underlying stream (little
    /// endian)
    pub fn read_u32(&mut self) -> impl Future<Output = io::Result<u32>> + '_ {
        self.inner.read_u32_le()
    }

    /// Read `size` bytes from the socket without buffer
    /// pre-filling.
    pub async fn read_buffer(&mut self, size: usize) -> io::Result<Vec<u8>> {
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

    /// Reads variadic length of payload from the stream
    pub fn read_length(
        &mut self,
        flags: u8,
    ) -> impl Future<Output = io::Result<u16>> + '_ {
        self.read_variadic(flags, flags::SHORT)
    }

    /// Reads variadic client id from the stream
    pub fn read_client_id(
        &mut self,
        flags: u8,
    ) -> impl Future<Output = io::Result<u16>> + '_ {
        self.read_variadic(flags, flags::SHORT_CLIENT)
    }

    /// Reads `u8` or `u16` from the stream, depending on
    /// flags.
    pub async fn read_variadic(
        &mut self,
        current_flags: u8,
        needed: u8,
    ) -> io::Result<u16> {
        if (current_flags & needed) == needed {
            self.read_u8().await.map(|o| o as u16)
        } else {
            self.read_u16().await
        }
    }
}

// Bufferization & creation related stuff

impl<R, D> MidReader<R, D>
where
    R: AsyncRead,
{
    /// Create buffered reader (wraps R with `BufReader<R>`
    /// with specified capacity)
    pub fn make_buffered(
        self,
        buffer_size: usize,
        decompressor: D,
    ) -> MidReader<BufReader<R>, D> {
        MidReader::new_buffered(self.inner, decompressor, buffer_size)
    }
}

impl<R, D> MidReader<BufReader<R>, D>
where
    R: AsyncRead,
{
    /// Create buffered version of the reader
    pub fn new_buffered(
        socket: R,
        decompressor: D,
        buffer_size: usize,
    ) -> Self {
        Self {
            inner: BufReader::with_capacity(buffer_size, socket),
            decompressor,
        }
    }

    /// Remove underlying buffer.
    ///
    /// WARNING: buffered data can be lost!
    pub fn unbuffer(self) -> MidReader<R, D> {
        MidReader {
            inner: self.inner.into_inner(),
            decompressor: self.decompressor,
        }
    }
}

impl<R, D> MidReader<R, D> {
    /// Get shared access to the underlying socket
    pub const fn socket(&self) -> &R {
        &self.inner
    }

    /// Get exclusive access to the underlying socket
    pub fn socket_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Simply create reader from the underlying socket type
    pub const fn new(socket: R, decompressor: D) -> Self {
        Self {
            inner: socket,
            decompressor,
        }
    }
}
