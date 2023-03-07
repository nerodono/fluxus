use std::{
    io,
    num::NonZeroUsize,
};

use galaxy_net_raw::packet_type::PacketType;
use galaxy_shrinker::error::DecompressError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("I/O Error: {0}")]
    Io(#[from] io::Error),

    #[error("Decompression error: {0}")]
    DecompressError(#[from] DecompressError),

    #[error("Got invalid rights bit-pattern: 0x{0:x}")]
    InvalidRights(u16),

    #[error("Unknown packet type: {type_}")]
    UnknownPacketType { type_: u8 },

    #[error("Unexpected packet {got}, expected: {expected}")]
    Unexpected {
        got: PacketType,
        expected: PacketType,
    },

    #[error(
        "Got too long buffer size for current configuration: {got} \
         > {expected}"
    )]
    TooLongBufferSize { expected: NonZeroUsize, got: usize },

    #[error("Could not retrieve decompression size")]
    NoDecompressionSize,

    #[error("Unknown protocol supplied: 0x{supplied:x}")]
    UnknownProtocol { supplied: u8 },

    #[error("Unknown error code received: 0x{code:x}")]
    UnknownErrorCode { code: u8 },
}
