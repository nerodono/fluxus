use galaxy_network::{
    raw::CompressionAlgorithm,
    shrinker::zstd::{
        ZStdCctx,
        ZStdDctx,
    },
};

use crate::config::CompressionConfig;

// TODO: Polymorphic compression
pub fn create_compressor_decompressor(
    cfg: &CompressionConfig,
) -> (ZStdCctx, ZStdDctx) {
    match cfg.algorithm {
        CompressionAlgorithm::ZStd => {
            (ZStdCctx::new(cfg.level), ZStdDctx::new())
        }
    }
}
