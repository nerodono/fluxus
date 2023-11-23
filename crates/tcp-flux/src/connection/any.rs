use std::net::SocketAddr;

use tokio::net::TcpStream;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    Flow { id: u64 },
    Master,
}

#[derive(Debug)]
pub struct AnyConnection {
    pub type_: ConnectionType,
    pub address: SocketAddr,
    pub socket: TcpStream,
}

#[rustfmt::skip]
impl ConnectionType {
    pub const MASTER_INT: u8 = b'm';
    pub const FLOW_INT: u8   = b's';
}
