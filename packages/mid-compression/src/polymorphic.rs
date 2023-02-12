use enum_dispatch::enum_dispatch;

use crate::{
    interface::*,
    zstd,
};

/// Polymorphic compression routines.
#[enum_dispatch]
pub enum PolyCompressor {
    ZStd(zstd::compressor::ZStdCctx),
}

/// Polymorphic decompression routines
#[enum_dispatch]
pub enum PolyDecompressor {
    ZStd(zstd::decompressor::ZStdDctx),
}

impl PolyDecompressor {
    pub fn zstd() -> Self {
        Self::ZStd(zstd::decompressor::ZStdDctx::new())
    }
}

impl PolyCompressor {
    pub fn zstd(level: u8) -> Self {
        Self::ZStd(zstd::compressor::ZStdCctx::new(level))
    }
}
