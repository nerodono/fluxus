use std::io;

use cocoa_purity::map_enum;
use flux_tcp::{
    error::{
        ControlReadError,
        FlowReadError,
    },
    raw::NetworkError,
};
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

use crate::communication::slave::SlaveCommand;

pub type NonCriticalResult<T> = Result<T, NonCriticalError>;
pub type ProcessResult<T> = Result<T, ProcessError>;
pub type FlowProcessResult<T> = Result<T, FlowProcessError>;

#[derive(Debug, Error)]
pub enum FlowProcessError {
    #[error("flow read error: {0}")]
    Flow(#[from] FlowReadError),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("compressed buffer was too long")]
    CompressedBufferTooLong,

    #[error("slave channel was closed")]
    SlaveWasClosed,

    #[error("disconnected during forward")]
    DisconnectedDuringForward,

    #[error("failed to decompress forward packet")]
    FailedToDecompress,
}

#[derive(Debug, Error)]
#[map_enum(NetworkError)]
pub enum NonCriticalError {
    #[error("access denied: {0}")]
    AccessDenied(&'static str),

    #[error("wrong password supplied")]
    WrongPassword = AccessDenied,

    #[error("functionality ({0}) was disabled")]
    Disabled(&'static str),

    #[error("failed to bind 0.0.0.0:{port}: {error}")]
    FailedToBindAddress {
        port: u16,

        #[source]
        error: io::Error,
    },
}

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("{0}")]
    Control(#[from] ControlReadError),

    #[error("non-critical error: {0}")]
    NonCritical(#[from] NonCriticalError),
}

impl From<SendError<SlaveCommand>> for FlowProcessError {
    fn from(_: SendError<SlaveCommand>) -> Self {
        Self::SlaveWasClosed
    }
}
