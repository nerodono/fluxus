use std::io;

use tcp_flux::error::PktBaseReadError;
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
