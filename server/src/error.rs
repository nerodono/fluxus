use std::io;

use cocoa_purity::map_enum;
use galaxy_network::{
    error::ReadError,
    raw::{
        ErrorCode,
        Protocol,
    },
};
use integral_enum::integral_enum;
use thiserror::Error;

pub type NonCriticalResult<T> = Result<T, NonCriticalError>;
pub type ProcessResult<T> = Result<T, ProcessError>;

#[integral_enum]
#[derive(Error)]
pub enum PermitSendError {
    #[error("The channel is closed")]
    Closed,
}

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

    #[error("Selected protocol is unavailable ({0:?})")]
    ProtocolIsUnavailable(Protocol) = Unavailable,

    #[error("Not enough rights for the {0:?} protocol")]
    NotEnoughRightsForProtocol(Protocol) = AccessDenied,

    #[error("User has no access to select port {0} (protocol = {1:?})")]
    NoAccessToSelectPort(u16, Protocol) = AccessDenied,

    #[error("Not found client with id {id} (chan_closed? {chan_closed})")]
    ClientIsNotFound { id: u16, chan_closed: bool } = ClientDoesNotExists,

    #[error("Failed to bind port {port} for protocol {protocol:?}: {error}")]
    FailedToBindAddress {
        error: io::Error,
        port: u16,
        protocol: Protocol,
    },

    #[error("No server was created")]
    NoServerWasCreated,
}
