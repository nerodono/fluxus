use integral_enum::IntegralEnum;
use thiserror::Error;

#[derive(IntegralEnum, Error)]
pub enum CompressError {
    #[error("Suppleid buffer is too short")]
    TooShortBuffer,

    #[error("Invalid compression level supplied")]
    InvalidLevel,
}

#[derive(IntegralEnum, Error)]
pub enum DecompressError {
    #[error("Insufficient buffer size for decompression")]
    InsufficientBuffer,

    #[error("Invalid data supplied (possibly wrong format)")]
    InvalidData,
}

#[derive(IntegralEnum, Error)]
pub enum SizeRetrievalError {
    #[error("Invalid buffer supplied")]
    InvalidData,

    #[error(
        "Decompression backend does not supports data size retrieval"
    )]
    NotSupported,
}
