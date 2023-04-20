use std::io;

use galaxy_network::{
    error::ReadError,
    raw::ErrorCode,
};
use integral_enum::integral_enum;
use thiserror::Error;

pub type ProcessResult<T> = Result<T, ProcessError>;
pub type NonCriticalResult<T> = Result<T, NonCriticalError>;

#[cfg(feature = "http")]
#[integral_enum]
#[derive(Error)]
pub enum HttpEndpointCreationError {
    #[error("Endpoint is already occupied")]
    Occupied,
}

#[derive(Debug, Error)]
pub enum NonCriticalError {
    #[error("Requested feature was disabled")]
    FeatureIsDisabled,

    #[error("No server was created")]
    NoServer,

    #[error("No client associated with that ID")]
    ClientDoesNotExists,
}

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("Non-critical error reported: {0}")]
    NonCritical(#[from] NonCriticalError),

    #[error("{0}")]
    Read(#[from] ReadError),
}

#[integral_enum]
#[derive(Error)]
pub enum PermitSendError {
    #[error("The channel is closed")]
    Closed,
}

#[integral_enum]
#[derive(Error)]
pub enum SendCommandError {
    #[error("The channel is closed")]
    Closed,

    #[error("Client not found")]
    ClientNotFound,
}

impl From<io::Error> for ProcessError {
    fn from(value: io::Error) -> Self {
        Self::Read(ReadError::Io(value))
    }
}

impl From<NonCriticalError> for ErrorCode {
    fn from(value: NonCriticalError) -> Self {
        match value {
            NonCriticalError::ClientDoesNotExists => {
                Self::ClientDoesNotExists
            }
            NonCriticalError::FeatureIsDisabled => Self::Unavailable,
            NonCriticalError::NoServer => Self::NoServerWasCreated,
        }
    }
}
