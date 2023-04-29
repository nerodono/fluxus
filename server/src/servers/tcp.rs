use rustc_hash::FxHashMap;
use tokio::sync::mpsc;

use crate::{
    data::commands::tcp::TcpSlaveCommand,
    error::{
        NonCriticalError,
        NonCriticalResult,
    },
    utils::return_id::ReturnId,
};

type Channel = mpsc::UnboundedSender<TcpSlaveCommand>;

#[derive(Default)]
pub struct TcpServer {
    clients: FxHashMap<u16, Channel>,
}

impl TcpServer {
    pub fn insert_client(&mut self, id: u16, channel: Channel) {
        self.clients.insert(id, channel);
    }

    pub fn send_command(
        &self,
        id: u16,
        command: TcpSlaveCommand,
    ) -> NonCriticalResult<()> {
        let channel = self.clients.get(&id).ok_or(
            NonCriticalError::ClientIsNotFound {
                id,
                chan_closed: false,
            },
        )?;
        channel.send(command).map_err(|_| {
            NonCriticalError::ClientIsNotFound {
                id,
                chan_closed: true,
            }
        })
    }

    pub fn remove_client(
        &mut self,
        id: u16,
    ) -> NonCriticalResult<ReturnId<u16, Channel>> {
        self.clients
            .remove(&id)
            .map(|channel| ReturnId::new(id, channel))
            .ok_or(NonCriticalError::ClientIsNotFound {
                id,
                chan_closed: true,
            })
    }
}
