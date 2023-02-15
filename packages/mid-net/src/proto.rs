use integral_enum::IntegralEnum;
use thiserror::Error;

#[derive(IntegralEnum)]
pub enum Protocol {
    Tcp = 0,
    Udp = 1,
}

#[derive(IntegralEnum)]
pub enum PacketType {
    Failure = 0,
    Ping = 1,

    Connect = 2,
    Forward = 3,
    Disconnect = 4,

    CreateServer = 5,

    Authorize = 6,
    UpdateRights = 7,
}

#[derive(IntegralEnum, Error)]
pub enum ProtocolError {
    #[error("Unknown packet sent")]
    UnknownPacket = 0,

    #[error("Unexpected packet for that side")]
    UnexpectedPacket = 1,

    #[error("Functionality currently is not implemented")]
    Unimplemented = 2,

    #[error("You don't have access to this functionality")]
    AccessDenied = 3,

    #[error("Functionality is disabled")]
    Disabled = 4,

    #[error("Failed to create TCP listener")]
    FailedToCreateListener = 5,

    #[error("Failed to retrieve listening port")]
    FailedToRetrievePort = 6,

    #[error("The server is already created")]
    AlreadyCreated = 7,

    #[error("The server is not created")]
    ServerIsNotCreated = 8,

    #[error("Client does not exists")]
    ClientDoesNotExists = 9,
}
