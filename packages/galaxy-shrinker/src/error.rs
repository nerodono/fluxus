use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompressError {
    #[error("Insufficient buffer size supplied")]
    InsufficientBuffer,
}

#[derive(Debug, Error)]
pub enum DecompressError {
    #[error("Too short buffer supplied")]
    TooShortBuffer,

    #[error("Invalid data supplied")]
    InvalidData,

    #[error(
        "Allocation was declined by the underlying allocator \
         function"
    )]
    Declined,
}
