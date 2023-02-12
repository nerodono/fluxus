use enum_dispatch::enum_dispatch;

use crate::{
    interface::*,
    zstd,
};

/// Polymorphic compression routines.
#[enum_dispatch]
pub enum PolyCompressor {
    ZStd(zstd::compressor::ZStdCctx),
    Unit(()),
}

/// Polymorphic decompression routines
#[enum_dispatch]
pub enum PolyDecompressor {
    ZStd(zstd::decompressor::ZStdDctx),
    Unit(()),
}

impl PolyDecompressor {
    /// Create unit decompressor. Will panic on every
    /// decompression
    pub const fn unit() -> Self {
        Self::Unit(())
    }

    /// Create ZStandard decompressor
    pub fn zstd() -> Self {
        Self::ZStd(zstd::decompressor::ZStdDctx::new())
    }
}

impl PolyCompressor {
    /// Create unit compressor. Will panic on every
    /// compression
    pub const fn unit() -> Self {
        Self::Unit(())
    }

    /// Create ZStandard decompressor with supplied level
    pub fn zstd(level: u8) -> Self {
        Self::ZStd(zstd::compressor::ZStdCctx::new(level))
    }
}
