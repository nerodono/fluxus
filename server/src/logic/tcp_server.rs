use std::sync::Arc;

use idpool::interface::IdPool;
use rustc_hash::FxHashMap;
use tokio::sync::{
    mpsc::{
        unbounded_channel,
        UnboundedReceiver,
        UnboundedSender,
    },
    Mutex,
};

use super::command::{
    MasterCommand,
    SlaveCommand,
};
use crate::error::{
    ChanSendError,
    CreateClientError,
};

pub type TcpIdPool = Mutex<Box<dyn IdPool<Id = u16> + Send>>;

pub struct TcpProxyServer {
    pub send_chan: UnboundedSender<MasterCommand>,
    pub(crate) recv_chan: UnboundedReceiver<MasterCommand>,

    clients: FxHashMap<u16, UnboundedSender<SlaveCommand>>,
    pub pool: Arc<TcpIdPool>,
}

impl TcpProxyServer {
    pub async fn create_client(
        &mut self,
        id: u16,
        chan: UnboundedSender<SlaveCommand>,
    ) -> Result<(), CreateClientError> {
        self.clients.insert(id, chan);
        Ok(())
    }

    pub async fn send_to(
        &self,
        id: u16,
        command: SlaveCommand,
    ) -> Result<(), ChanSendError> {
        match self.clients.get(&id) {
            Some(chan) => chan
                .send(command)
                .map_err(|_| ChanSendError::ChannelIsClosed),

            None => Err(ChanSendError::IdDoesNotExists),
        }
    }

    pub fn new(pool: TcpIdPool) -> Self {
        let (send_chan, recv_chan) = unbounded_channel();
        Self {
            pool: Arc::new(pool),
            clients: Default::default(),
            send_chan,
            recv_chan,
        }
    }
}
