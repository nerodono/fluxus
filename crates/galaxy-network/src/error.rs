#[cfg(feature = "tokio")]
use std::io;

use thiserror::Error;

#[cfg(feature = "tokio")]
#[derive(Debug, Error)]
pub enum ReadError {
    #[error("I/O Error: {0}")]
    Io(#[from] io::Error),

    #[error("Got unknown packet type")]
    UnknownPacket,

    #[error("Failed to retrieve payload uncompressed size")]
    FailedToRetrieveUncompressedSize,

    #[error("Failed to decompress data")]
    FailedToDecompress,
}
