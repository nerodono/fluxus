use std::io;

use cocoa_purity::map_enum;
use galaxy_network::{
    error::ReadError,
    raw::ErrorCode,
};
use thiserror::Error;

pub type NonCriticalResult<T> = Result<T, NonCriticalError>;
pub type ProcessResult<T> = Result<T, ProcessError>;

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("Read error: {0}")]
    Read(#[from] ReadError),

    #[error("I/O Error: {}", 1)]
    Io(#[from] io::Error),

    #[error("Non-critical error: {0}")]
    NonCritical(#[from] NonCriticalError),
}

#[map_enum(ErrorCode)]
#[derive(Debug, Error)]
pub enum NonCriticalError {
    #[error("Requested feature ({0}) is unavailable")]
    FeatureIsUnavailable(&'static str) = Unavailable,

    #[error("Requested functionality was not implemented")]
    Unimplemented(&'static str),

    #[error(
        "Incorrect password for universal password authorization supplied"
    )]
    IncorrectUniversalPassword = AccessDenied,
}
