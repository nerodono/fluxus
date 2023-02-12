///! # Middleware compression
///!
///! This package is a set of compression utilities for the
/// `Middleware` protocol.
///!
///! Contains polymorphic compression/decompression
/// strategies without dynamic dispatch (refer to the
/// [`polymorphic`]).

/// # Compression/decompression errors
///
/// Contains types for error reporting. Basically it is
/// __almost__ detailed description of what's gone wrong.
pub mod error;

/// # Compressor/decompressor interfaces
///
/// Every algorithm should implement either [`ICompressor`]
/// or [`IDecompressor`] to be used in [`polymorphic`]
/// decompression routines.
pub mod interface;

/// # Polymorphic compression/decompression
///
/// Can use plenty of algorithms without dynamic dispatch.
pub mod polymorphic;

// Algorithms

/// # Deflate compression algorithm
pub mod deflate;

/// # ZStandard compression algorithm
pub mod zstd;

#[cold]
pub(crate) fn cold() {}
