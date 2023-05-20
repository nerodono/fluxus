use std::io;

use thiserror::Error;

pub type ControlReadResult<T> = Result<T, ControlReadError>;
pub type FlowReadResult<T> = Result<T, FlowReadError>;

#[derive(Debug, Error)]
pub enum ControlReadError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("invalid rights bits: 0x{0:x}")]
    InvalidRights(u8),

    #[error("got unknown packet type: 0x{0:x}")]
    UnknownPacket(u8),

    #[error("client disconnected during read")]
    Disconnected,
}

#[derive(Debug, Error)]
pub enum FlowReadError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("client disconnected during read")]
    Disconnected,
}
