use tokio::io::{
    AsyncRead,
    BufReader,
};

use super::scoped::{
    ClientReader,
    CommonReader,
    RawReader,
    ServerReader,
};
use crate::error::ReadError;

pub type ReadResult<T> = Result<T, ReadError>;

/// Read side of the stream
pub struct GalaxyReader<R, D> {
    pub(crate) stream: R,
    pub(crate) decompressor: D,
}

impl<R, D> GalaxyReader<R, D> {
    /// Get `raw` scope of the reader.
    pub fn raw(&mut self) -> RawReader<'_, R, D> {
        RawReader { stream: self }
    }

    /// Get `common` scope of the reader.
    pub fn common(&mut self) -> CommonReader<'_, R, D> {
        CommonReader { raw: self.raw() }
    }

    /// Get `client` scope of the reader.
    pub fn client(&mut self) -> ClientReader<'_, R, D> {
        ClientReader { raw: self.raw() }
    }

    /// Get `server` scope of the reader.
    pub fn server(&mut self) -> ServerReader<'_, R, D> {
        ServerReader { raw: self.raw() }
    }
}

impl<R, D> GalaxyReader<BufReader<R>, D>
where
    R: AsyncRead,
{
    /// Same as [`GalaxyReader::new`], but wraps stream into
    /// buffered reader.
    pub fn new_buffered(
        stream: R,
        decompressor: D,
        buffer_size: usize,
    ) -> Self {
        Self {
            stream: BufReader::with_capacity(buffer_size, stream),
            decompressor,
        }
    }

    /// Unwraps stream from the buffered reader.
    ///
    /// Note: buffered data will be lost, so consider the
    /// [`GalaxyReader::buffer`] method to handle already
    /// buffered pile of data.
    pub fn unwrap_buffer(self) -> GalaxyReader<R, D> {
        GalaxyReader {
            stream: self.stream.into_inner(),
            decompressor: self.decompressor,
        }
    }

    /// Gets buffer from the underlying buffered reader
    pub fn buffer(&self) -> &[u8] {
        self.stream.buffer()
    }
}

impl<R, D> GalaxyReader<R, D> {
    /// Replaces current decompressor with new one
    pub fn with_decompressor<Nd>(
        self,
        new_decompressor: Nd,
    ) -> GalaxyReader<R, Nd> {
        GalaxyReader {
            stream: self.stream,
            decompressor: new_decompressor,
        }
    }

    /// Creates protocol read side from the stream and
    /// decompressor.
    pub const fn new(stream: R, decompressor: D) -> Self {
        Self {
            stream,
            decompressor,
        }
    }
}
