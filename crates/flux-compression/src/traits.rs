use std::num::NonZeroUsize;

use enum_dispatch::enum_dispatch;

#[allow(clippy::wildcard_imports)]
use crate::polymorphic::*;

#[derive(Debug, Clone, Copy)]
pub struct CompressionError;

#[enum_dispatch]
pub trait Compressor: Send {
    /// Tries to compress `src` bytes into the capacity of
    /// `dst` vector.
    ///
    /// on `Ok` return `dst` vector will be exactly of
    /// compressed bytes size length.
    fn try_compress_into(
        &mut self,
        src: &[u8],
        dst: &mut Vec<u8>,
    ) -> Result<(), CompressionError>;
}

#[enum_dispatch]
pub trait Decompressor: Send {
    /// Tries to decompress `src` buffer into newly
    /// allocated buffer.
    ///
    /// Returns [`None`] if:
    /// - `size_predicate(decompressed_size)` returns
    ///   [`false`]
    /// - decompressed size could not be retrieved
    /// - underlying decompression function returned an
    ///   error
    fn try_decompress(
        &mut self,
        src: &[u8],
        size_predicate: impl FnOnce(NonZeroUsize) -> bool,
    ) -> Option<Vec<u8>>;
}
