use std::num::NonZeroU16;

use galaxy_net_raw::related::Protocol;

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
