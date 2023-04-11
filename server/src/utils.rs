use galaxy_network::{
    raw::CompressionAlgorithm,
    shrinker::zstd::{
        ZStdCctx,
        ZStdDctx,
    },
};

use crate::config::CompressionConfig;

// FIXME: Polymorphic compressor / decompressor
pub fn create_compressor_decompressor(
    cfg: &CompressionConfig,
) -> (ZStdCctx, ZStdDctx) {
    match cfg.algorithm {
        CompressionAlgorithm::ZStd => {
            (ZStdCctx::new(cfg.level), ZStdDctx::new())
        }
    }
}
