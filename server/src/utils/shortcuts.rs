use std::io;

use galaxy_network::raw::Protocol;

use crate::error::{
    NonCriticalError,
    NonCriticalResult,
};

pub fn convert_io<T>(
    result: io::Result<T>,
    error_fn: impl FnOnce(io::Error) -> NonCriticalError,
) -> NonCriticalResult<T> {
    result.map_err(error_fn)
}

pub fn assert_bound<T>(
    port: u16,
    protocol: Protocol,
    result: io::Result<T>,
) -> NonCriticalResult<T> {
    convert_io(result, |error| NonCriticalError::FailedToBindAddress {
        error,
        port,
        protocol,
    })
}
