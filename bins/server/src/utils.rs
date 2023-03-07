use galaxy_net::raw::related::CompressionMethod;

use crate::config::{
    CompressionEntry,
    CompressionMethodEntry,
};

pub fn extract_used_compression(
    compression: &CompressionEntry,
) -> (CompressionMethod, &CompressionMethodEntry) {
    (
        compression.use_,
        match compression.use_ {
            CompressionMethod::ZStd => &compression.zstd,
        },
    )
}
