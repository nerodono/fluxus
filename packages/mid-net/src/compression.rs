use integral_enum::IntegralEnum;
#[cfg(feature = "serde")]
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ForwardCompression {
    NoCompression,
    Compress { with_threshold: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DecompressionConstraint {
    Max(usize),
    MaxSizeMultiplier(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DecompressionStrategy {
    ConstrainedConst { constraint: DecompressionConstraint },
    Unconstrained,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CompressionStatus {
    Uncompressed,
    Compressed { before: usize, after: usize },
}

#[derive(IntegralEnum)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum CompressionAlgorithm {
    #[cfg_attr(feature = "serde", serde(rename = "zstd"))]
    ZStd = 0,

    #[cfg_attr(feature = "serde", serde(rename = "deflate"))]
    Deflate = 1,
}

pub use mid_compression as impl_;
