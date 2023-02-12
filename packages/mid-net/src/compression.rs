use integral_enum::IntegralEnum;
#[cfg(feature = "serde")]
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForwardCompression {
    NoCompression,
    Compress { with_threshold: usize },
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
    ZStd = 0,
    Deflate = 1,
}
