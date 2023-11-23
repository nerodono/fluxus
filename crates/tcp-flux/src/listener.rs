use std::io;

use tokio::{
    io::AsyncReadExt,
    net::{
        TcpListener,
        ToSocketAddrs,
    },
};

use crate::{
    connection::any::{
        AnyConnection,
        ConnectionType,
    },
    error::AcceptError,
};

pub struct Listener {
    handle: TcpListener,
}

impl Listener {
    pub const fn inner_ref(&self) -> &TcpListener {
        &self.handle
    }

    pub async fn next_connection(&self) -> Result<AnyConnection, AcceptError> {
        let (mut socket, address) = self.handle.accept().await?;
        let prot_int = socket.read_u8().await?;

        let conn_type = match prot_int {
            ConnectionType::FLOW_INT => {
                let flow_id = socket.read_u64_le().await?;
                ConnectionType::Flow { id: flow_id }
            }
            ConnectionType::MASTER_INT => ConnectionType::Master,

            _ => return Err(AcceptError::WrongProtocol(prot_int)),
        };

        Ok(AnyConnection {
            type_: conn_type,
            address,
            socket,
        })
    }

    pub async fn bind(address: impl ToSocketAddrs) -> io::Result<Self> {
        let handle = TcpListener::bind(address).await?;
        Ok(Self { handle })
    }
}
