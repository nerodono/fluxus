use std::io;

use tcp_flux::{
    error::PktBaseReadError,
    types::error_code::ErrorCode,
};
use thiserror::Error;

use crate::error::{
    CriticalError,
    NonCriticalError,
};

pub type TcpFluxResult<T> = Result<T, TcpFluxError>;

#[derive(Debug, Error)]
pub enum TcpFluxError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("{0}")]
    BaseReadError(#[from] PktBaseReadError),

    #[error("non-critical error: {0}")]
    NonCritical(#[from] NonCriticalError),

    #[error("critical error: {0}")]
    Critical(#[from] CriticalError),
}

pub const fn convert_critical(error: CriticalError) -> ErrorCode {
    use CriticalError as C;
    use ErrorCode as E;

    match error {
        C::UnexpectedPacket => E::UnexpectedPacket,
    }
}

pub const fn convert_non_critical(error: NonCriticalError) -> ErrorCode {
    use ErrorCode as E;
    use NonCriticalError as N;

    match error {
        N::FailedToAuthenticate => E::AuthenticationFailure,
        N::AccessDenied => E::AccessDenied,
    }
}
