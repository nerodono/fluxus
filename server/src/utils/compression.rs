use galaxy_network::{
    raw::CompressionAlgorithm,
    shrinker::zstd::{
        ZStdCctx,
        ZStdDctx,
    },
};

use crate::config::CompressionConfig;

/// Creates compressor/decompressor using the provided
/// config
///
/// # TODO
///
/// Polymorphic compressor / decompressor, currently
/// function returns only ``ZStd`` compressor/decompressor
pub fn create_compressor_decompressor(
    cfg: &CompressionConfig,
) -> (ZStdCctx, ZStdDctx) {
    match cfg.algorithm {
        CompressionAlgorithm::ZStd => {
            (ZStdCctx::new(cfg.level), ZStdDctx::new())
        }
    }
}
