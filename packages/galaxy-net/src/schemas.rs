use std::num::NonZeroU16;

use bitflags::bitflags;
use galaxy_net_raw::related::{
    CompressionMethod,
    Protocol,
};

bitflags! {
    pub struct Permissions: u16 {
        const CAN_CREATE_TCP = 1 << 0;
        const CAN_SELECT_TCP = 1 << 1;
    }
}

pub struct PingResponseDescriptor<'a> {
    pub server_name: &'a str,
    pub compression_method: CompressionMethod,
    pub compression_level: u8,
    pub compression_threshold: u16,
    pub read_buffer: usize,
}

/// Describes authorization request
#[derive(Debug)]
pub struct AuthRequestDescriptor {
    pub universal_password: String,
}

/// Describes the forward packet.
#[derive(Debug)]
pub struct ForwardPacketDescriptor {
    pub client_id: u16,
    pub buffer: Vec<u8>,
}

/// Describes the started server.
///
/// Note: [`None`] value of the `at_port` means that it can
/// be infered from context (e.g. requested specific port
/// from server explicitly).
#[derive(Debug)]
pub struct StartedServerDescriptor {
    pub at_port: Option<NonZeroU16>,
}

/// Describes the server create request.
#[derive(Debug, Clone)]
pub struct ServerDescriptor {
    pub protocol: Protocol,
    pub port: Option<NonZeroU16>,
}
