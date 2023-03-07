use std::num::NonZeroUsize;

use super::zstd::{
    cctx::*,
    dctx::*,
};
use crate::interface::*;

/// Polymorphic context for the compressor
pub enum PolymorphicCctx {
    ZStd(ZStdCctx),
}

/// Polymorphic context for the decompressor
pub enum PolymorphicDctx {
    ZStd(ZStdDctx),
}

impl PolymorphicDctx {
    /// Create ZStd decompressor
    pub fn zstd() -> Self {
        Self::ZStd(ZStdDctx::new())
    }
}

impl PolymorphicCctx {
    /// Create ZStd compressor
    pub fn zstd(level: NonZeroUsize) -> Self {
        // TODO: check available compression levels
        Self::ZStd(ZStdCctx::new(level.try_into().unwrap()))
    }
}

impl Decompressor for PolymorphicDctx {
    fn try_get_size(
        &self,
        of: &[u8],
    ) -> Option<std::num::NonZeroUsize> {
        match self {
            Self::ZStd(dctx) => dctx.try_get_size(of),
        }
    }

    fn try_decompress<F>(
        &mut self,
        src: &[u8],
        buffer_allocator: F,
    ) -> Result<Vec<u8>, crate::error::DecompressError>
    where
        F: FnOnce(Option<std::num::NonZeroUsize>) -> Option<Vec<u8>>,
    {
        match self {
            Self::ZStd(dctx) => {
                dctx.try_decompress(src, buffer_allocator)
            }
        }
    }
}

impl Compressor for PolymorphicCctx {
    fn try_compress(
        &mut self,
        src: &[u8],
        dst: &mut Vec<u8>,
    ) -> Result<usize, crate::error::CompressError> {
        match self {
            Self::ZStd(cctx) => cctx.try_compress(src, dst),
        }
    }
}
