use bitflags::bitflags;
use integral_enum::integral_enum;
use thiserror::Error;

#[rustfmt::skip]
#[integral_enum(u8)]
pub enum PacketType {
    Error = 0,
    Ping  = 1,

    CreateServer = 2,
    Connect = 3,
    Forward = 4,
    Disconnect = 5,

    AuthorizePassword = 6,
    UpdateRights = 7,
}

#[integral_enum(u8)]
pub enum Protocol {
    Tcp = 0,
    Udp = 1,
    Http = 2,
}

#[derive(Debug, Clone, Copy)]
pub struct Packet {
    pub type_: PacketType,
    pub flags: PacketFlags,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[integral_enum(u8)]
pub enum CompressionAlgorithm {
    ZStd = 0,
}

#[integral_enum(u8)]
#[derive(Error)]
pub enum ErrorCode {
    #[error("Unknown command sent")]
    UnknownCommand = 0,

    #[error("This kind of packet is unsupported")]
    Unsupported = 1,

    #[error("Requested functionality is not implemented")]
    Unimplemented = 2,

    #[error("You don't have access to requested functionality")]
    AccessDenied = 3,

    #[error("Failed to bind to requested address")]
    FailedToBindAddress = 4,

    #[error("No server was created")]
    NoServerWasCreated = 5,

    #[error("Client with that ID does not exists")]
    ClientDoesNotExists = 6,

    #[error("Failed to decompress payload")]
    FailedToDecompress = 7,

    #[error("Too long buffer size supplied")]
    TooLongBuffer = 8,

    #[error("Invalid command for that server type")]
    InvalidServerType = 9,

    #[error("Server was stopped")]
    ServerStopped = 10,
}

impl From<Packet> for u8 {
    fn from(value: Packet) -> Self {
        value.encode()
    }
}

impl Packet {
    pub const fn id(type_: PacketType) -> Self {
        Self::new(type_, PacketFlags::empty())
    }

    pub const fn new(type_: PacketType, flags: PacketFlags) -> Self {
        Self { type_, flags }
    }
}

impl Packet {
    pub const fn encode(self) -> u8 {
        ((self.type_ as u8) << 3) | self.flags.bits()
    }

    #[inline]
    pub fn from_u8(u: u8) -> Option<Packet> {
        let flags = PacketFlags::from_bits(u & 0b111)?;
        PacketType::try_from(u >> 3)
            .map(|type_| Self::new(type_, flags))
            .ok()
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct PacketFlags: u8 {
        const SHORT        = 1 << 0;
        const SHORT_CLIENT = 1 << 1;
        const COMPRESSED   = 1 << 2;
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Rights: u8 {
        const CAN_CREATE_TCP      = 1 << 0;
        const CAN_SELECT_TCP_PORT = 1 << 1;

        const CAN_CREATE_UDP      = 1 << 2;
        const CAN_SELECT_UDP_PORT = 1 << 3;

        const CAN_CREATE_HTTP     = 1 << 4;
        const CAN_SELECT_PATH     = 1 << 5;
        const CAN_SELECT_DOMAIN   = 1 << 6;
    }
}

impl Rights {
    pub const fn can_create_server(self, protocol: Protocol) -> bool {
        match protocol {
            Protocol::Http => self.intersects(Rights::CAN_CREATE_HTTP),
            Protocol::Tcp => self.intersects(Rights::CAN_CREATE_TCP),
            Protocol::Udp => self.intersects(Rights::CAN_CREATE_UDP),
        }
    }

    pub const fn can_select_port(self, protocol: Protocol) -> bool {
        match protocol {
            Protocol::Http => false,
            Protocol::Tcp => self.intersects(Rights::CAN_SELECT_TCP_PORT),
            Protocol::Udp => self.intersects(Rights::CAN_SELECT_UDP_PORT),
        }
    }
}
