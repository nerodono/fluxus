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

    #[error("Unknown protocol supplied: 0x{supplied:x}")]
    UnknownProtocol { supplied: u8 },
}
