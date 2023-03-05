use std::{
    future::Future,
    io,
};

use tokio::io::{
    AsyncWrite,
    AsyncWriteExt,
    BufWriter,
};

/// Write side of the protocol.
pub struct GalaxyWriter<W, C> {
    pub(crate) stream: W,
    pub(crate) compressor: C,
}

impl<W, C> GalaxyWriter<W, C> {}

impl<W, C> GalaxyWriter<BufWriter<W>, C>
where
    W: AsyncWrite,
{
    /// Unwraps underlying stream from the [`BufWriter`].
    ///
    /// Note: consider calling [`GalaxyWriter::flush`]
    /// before unwrapping the buffer, otherwise buffered
    /// data can be lost.
    pub fn unwrap_buffer(self) -> GalaxyWriter<W, C> {
        GalaxyWriter {
            stream: self.stream.into_inner(),
            compressor: self.compressor,
        }
    }

    /// Creates write side with the buffering
    pub fn new_buffered(
        stream: W,
        compressor: C,
        buffer_size: usize,
    ) -> Self {
        Self {
            stream: BufWriter::with_capacity(buffer_size, stream),
            compressor,
        }
    }
}

impl<W, C> GalaxyWriter<BufWriter<W>, C>
where
    W: AsyncWrite + Unpin,
{
    /// Flushes buffer after `f(self)` finishes its work.
    pub async fn flush_scope<'a, F, R0, R>(
        &'a mut self,
        f: F,
    ) -> io::Result<R0>
    where
        F: for<'k> Fn(&'k mut Self) -> R,
        R: Future<Output = R0> + 'a,
    {
        let result = f(self).await;
        self.flush().await?;
        Ok(result)
    }

    /// Write all buffered data into the stream.
    pub fn flush(
        &mut self,
    ) -> impl Future<Output = io::Result<()>> + '_ {
        self.stream.flush()
    }
}

impl<W, C> GalaxyWriter<W, C> {
    /// Replace current compressor with the new one.
    pub fn with_compressor<Nc>(
        self,
        new_compressor: Nc,
    ) -> GalaxyWriter<W, Nc> {
        GalaxyWriter {
            stream: self.stream,
            compressor: new_compressor,
        }
    }

    /// Creates [`GalaxyWriter`] from the writer and
    /// compressor.
    pub const fn new(stream: W, compressor: C) -> Self {
        Self { stream, compressor }
    }
}
