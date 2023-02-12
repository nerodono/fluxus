use enum_dispatch::enum_dispatch;

/// Interface for the decompressor.
#[enum_dispatch(PolyDecompressor)]
pub trait IDecompressor {
    /// Tries to retrieve decompressed size of the buffer.
    fn try_decompressed_size(
        &self,
        of: &[u8],
    ) -> Result<usize, crate::error::SizeRetrievalError>;

    /// Tries to decompress supplied `buffer` into the `to`
    /// Vec.
    ///
    /// Maximum decompression size is assumed to be
    /// `Vec::capacity`
    fn try_decompress(
        &mut self,
        buffer: &[u8],
        to: &mut Vec<u8>,
    ) -> Result<usize, crate::error::DecompressError>;
}

/// Interface for the compressor.
#[enum_dispatch(PolyCompressor)]
pub trait ICompressor {
    /// Get compression level of this compressor.
    fn level(&self) -> usize;

    /// Set compression level of this compressor.
    fn set_level(&mut self, level: usize);

    /// Tries to compress buffer `buf` to the `preallocated`
    /// `Vec`.
    ///
    /// Maximum compressed size is assumed to be
    /// `Vec::capacity`.
    fn try_compress(
        &mut self,
        buf: &[u8],
        preallocated: &mut Vec<u8>,
    ) -> Result<std::num::NonZeroUsize, crate::error::CompressError>;

    /// Get backend's supported compression levels.
    fn supported_levels(&self) -> std::ops::RangeInclusive<usize>;
}
