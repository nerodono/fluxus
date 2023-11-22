use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PktBaseReadError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("invalid packet type: 0x{0:x}")]
    InvalidType(u8),
}

#[derive(Debug, Error)]
pub enum AcceptError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("wrong protocol selected: 0x{0:x}")]
    WrongProtocol(u8),
}
