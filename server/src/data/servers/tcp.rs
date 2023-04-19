use std::hash::BuildHasherDefault;

use idpool::interface::IdPool;
use rustc_hash::FxHashMap;
use tokio::sync::mpsc;

use crate::{
    data::{
        commands::tcp::TcpSlaveCommand,
        id_pool::IdPoolImpl,
    },
    error::SendCommandError,
};

pub struct TcpServer {
    pool: IdPoolImpl,
    clients: FxHashMap<u16, mpsc::UnboundedSender<TcpSlaveCommand>>,
}

impl TcpServer {
    /// # Panics
    ///
    /// Panics if `client_id` is not present in the server's
    /// map.
    pub async fn remove_channel(
        &mut self,
        client_id: u16,
    ) -> mpsc::UnboundedSender<TcpSlaveCommand> {
        self.pool.lock().await.return_id(client_id);
        self.clients.remove(&client_id).unwrap()
    }

    #[inline]
    pub fn send_command(
        &self,
        client_id: u16,
        command: TcpSlaveCommand,
    ) -> Result<(), SendCommandError> {
        self.clients.get(&client_id).map_or(
            Err(SendCommandError::ClientNotFound),
            |chan| {
                chan.send(command)
                    .map_err(|_| SendCommandError::Closed)
            },
        )
    }

    #[inline]
    pub fn insert_channel(
        &mut self,
        client_id: u16,
        channel: mpsc::UnboundedSender<TcpSlaveCommand>,
    ) {
        self.clients.insert(client_id, channel);
    }
}

impl TcpServer {
    pub fn new(id_pool: IdPoolImpl) -> Self {
        Self {
            pool: id_pool,
            clients: FxHashMap::with_capacity_and_hasher(
                8,
                BuildHasherDefault::default(),
            ),
        }
    }
}
