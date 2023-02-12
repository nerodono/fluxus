use integral_enum::IntegralEnum;
#[cfg(feature = "serde")]
use serde::{
    Deserialize,
    Serialize,
};

#[derive(IntegralEnum)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum CompressionAlgorithm {
    ZStd = 0,
    Deflate = 1,
}
