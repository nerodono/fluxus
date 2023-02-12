///! # Middleware network
///!
///! This package contains routines for the `Middleware`
/// protocol, such as:
/// - [`reader::MidReader`] - Protocol reader, consumes read
///   side of the socket
/// - [`writer::MidWriter`] - Protocol writer, consumes
///   write side of the socket
///
///! read/write sides can be obtained through the `split()`
///! function of the tokio's `TcpStream`

/// # Reader
///
/// Refer to the [`MidReader`]
pub mod reader;

/// # Writer
///
/// Refer to the [`MidWriter`]
pub mod writer;

pub mod compression;
pub mod utils;

/// # Protocol enums
pub mod proto;

/// # Middleware errors
pub mod error;

#[cfg(test)]
mod tests;
