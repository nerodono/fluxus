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

    #[error("Invalid rights bits")]
    InvalidRights { bits: u8 },

    #[error("Invalid compression algorithm: 0x{0:x}")]
    InvalidCompressionAlgorithm(u8),

    #[error("Invalid compression level: {0}")]
    InvalidCompressionLevel(u8),

    #[error("Invalid read buffer supplied: {0}")]
    InvalidReadBuffer(u16),

    #[error("Got invalid error code")]
    InvalidErrorCode,
}
