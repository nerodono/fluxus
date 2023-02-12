use std::io;

use thiserror::Error;

use crate::compression::DecompressionConstraint;

#[derive(Debug, Error)]
pub enum CompressedReadError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid compression data was read")]
    InvalidData,

    #[error("Specified constraint failed: {constraint:?}")]
    ConstraintFailed { constraint: DecompressionConstraint },
}
