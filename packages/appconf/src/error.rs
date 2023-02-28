use std::io;

use thiserror::Error;

/// Generic error enum. Used to represent any parse errors
/// during the loading routines
#[derive(Debug, Error)]
pub enum GenericError<T> {
    #[error("I/O Error: {0}")]
    Io(#[from] io::Error),

    #[error("Parse error: {0}")]
    Parser(T),
}
