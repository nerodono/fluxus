use galaxy_network::{
    descriptors::CompressionDescriptor,
    raw::CompressionAlgorithm,
    shrinker::zstd::{
        ZStdCctx,
        ZStdDctx,
    },
};

pub fn create_compressor_decompressor(
    response: &CompressionDescriptor,
) -> (ZStdCctx, ZStdDctx) {
    match response.algorithm {
        CompressionAlgorithm::ZStd => {
            (ZStdCctx::new(response.level), ZStdDctx::new())
        }
    }
}
