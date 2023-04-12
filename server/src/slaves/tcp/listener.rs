use std::{
    io,
    net::SocketAddr,
};

use tokio::net::TcpListener;

use crate::data::{
    command::erased::TcpPermit,
    idpool::IdPool,
};

pub async fn tcp_slave_listener(
    permit: TcpPermit,
    pool: IdPool,
    creator: SocketAddr,
    bound_addr: SocketAddr,
    listener: TcpListener,
) -> io::Result<()> {
    todo!()
}
