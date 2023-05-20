use flux_compression::{
    polymorphic::{
        PolyCctx,
        PolyDctx,
    },
    zstd::{
        ZStdCctx,
        ZStdDctx,
    },
};
use flux_tcp::raw::CompressionAlgorithm;

use crate::config::CompressionScope;

pub fn create_dctx(scope: &CompressionScope) -> PolyDctx {
    match scope.algorithm {
        CompressionAlgorithm::ZStd => PolyDctx::ZStd(ZStdDctx::new()),
    }
}

pub fn create_cctx(scope: &CompressionScope) -> PolyCctx {
    match scope.algorithm {
        CompressionAlgorithm::ZStd => {
            PolyCctx::ZStd(ZStdCctx::new(scope.level))
        }
    }
}
