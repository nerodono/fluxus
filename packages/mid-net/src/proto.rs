use integral_enum::IntegralEnum;

#[derive(IntegralEnum)]
pub enum PacketType {
    Failure = 0,
    Ping = 1,

    Connect = 2,
    Forward = 3,
    Disconnect = 4,
}

#[derive(IntegralEnum)]
pub enum ProtocolError {
    UnknownPacket = 0,
}
