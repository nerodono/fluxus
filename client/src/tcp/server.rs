use std::sync::Arc;

use rustc_hash::FxHashMap;
use tokio::sync::{
    mpsc,
    RwLock,
};

use super::command::{
    MasterCommand,
    SlaveCommand,
};
use crate::error::CommandSendError;

pub struct ServerStats {}

pub struct TcpRemoteServer {
    pub chan_tx: mpsc::UnboundedSender<MasterCommand>,
    pub chan_rx: mpsc::UnboundedReceiver<MasterCommand>,

    // TODO: add server statistics gathering
    pub stats: Arc<RwLock<ServerStats>>,

    slaves: FxHashMap<u16, mpsc::UnboundedSender<SlaveCommand>>,
}

impl TcpRemoteServer {
    pub fn new_slave(
        &mut self,
        id: u16,
        chan: mpsc::UnboundedSender<SlaveCommand>,
    ) {
        self.slaves.insert(id, chan);
    }

    #[inline]
    pub fn just_remove_client(&mut self, client_id: u16) {
        let _ = self.slaves.remove(&client_id);
    }

    pub fn remove_client(
        &mut self,
        client_id: u16,
    ) -> Result<(), CommandSendError> {
        self.send_command(client_id, SlaveCommand::Disconnect)?;
        let _ = self.slaves.remove(&client_id);

        Ok(())
    }

    pub fn send_command(
        &self,
        to: u16,
        command: SlaveCommand,
    ) -> Result<(), CommandSendError> {
        self.slaves
            .get(&to)
            .ok_or(CommandSendError::NoSuchClient)
            .and_then(|chan| {
                chan.send(command)
                    .map_err(|_| CommandSendError::Closed)
            })
    }
}

impl Default for TcpRemoteServer {
    fn default() -> Self {
        let (chan_tx, chan_rx) = mpsc::unbounded_channel();
        Self {
            stats: Arc::new(RwLock::new(ServerStats {})),
            chan_tx,
            chan_rx,
            slaves: FxHashMap::with_capacity_and_hasher(
                8,
                Default::default(),
            ),
        }
    }
}
