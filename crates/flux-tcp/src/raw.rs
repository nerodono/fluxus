use bitflags::bitflags;
use integral_enum::integral_enum;
use thiserror::Error;

#[integral_enum(u8)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum CompressionAlgorithm {
    ZStd = 0,
}

#[integral_enum(u8)]
pub enum ConnectProtocol {
    Flow = 0,
    Control = 1,
}

#[integral_enum(u8)]
pub enum PacketType {
    Error = 0,
    Hello = 1,

    Connected = 3,
    UpdateRights = 4,

    AuthorizePassword = 5,

    CreateTcpServer = 6,
    CreateHttpServer = 7,
}

#[derive(Error)]
#[integral_enum(u8)]
pub enum NetworkError {
    #[error("unknown packet type")]
    UnknownPacket = 0,

    #[error("compressed buffer is too long")]
    TooLongCompressedBuffer = 1,

    #[error("access denied")]
    AccessDenied = 2,

    #[error("requested functionality was not implemented")]
    Unimplemented = 3,

    #[error("requested functionality was disabled")]
    Disabled = 4,

    #[error("server was shut down")]
    Shutdown = 5,

    #[error("failed to bind requested address")]
    FailedToBindAddress = 6,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct FlowPacketFlags: u8 {
        const COMPRESSED = 1 << 0;
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Rights: u8 {
        const CREATE_TCP       = 1 << 0;
        const SELECT_TCP_PORT  = 1 << 1;

        const CREATE_HTTP      = 1 << 2;
        const SELECT_HTTP_HOST = 1 << 3;
    }
}
