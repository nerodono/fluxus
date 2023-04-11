use std::{
    net::SocketAddr,
    sync::Arc,
};

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

use crate::{
    data::{
        command::tcp::{
            TcpMasterCommand,
            TcpSlaveCommand,
        },
        shutdown_token::{
            shutdown_token,
            ShutdownListener,
            ShutdownTrigger,
        },
    },
    error::{
        ChanSendError,
        UnmapClientError,
    },
};

pub type TcpIdPool = Arc<Mutex<dyn IdPool<Id = u16> + Send>>;

pub struct TcpProxyServer {
    pub send_chan: UnboundedSender<TcpMasterCommand>,
    pub(crate) recv_chan: UnboundedReceiver<TcpMasterCommand>,

    clients: FxHashMap<u16, UnboundedSender<TcpSlaveCommand>>,
    pub pool: Arc<TcpIdPool>,

    address: SocketAddr,
    creator: SocketAddr,
    _shutdown_token: ShutdownTrigger,
}

impl TcpProxyServer {
    pub fn unmap_client(
        &mut self,
        id: u16,
    ) -> Result<(), UnmapClientError> {
        self.clients
            .remove(&id)
            .ok_or(UnmapClientError::ClientDoesNotExists)
            .map(|_| ())
    }

    pub fn map_client(
        &mut self,
        id: u16,
        chan: UnboundedSender<TcpSlaveCommand>,
    ) {
        self.clients.insert(id, chan);
    }

    pub fn send_to(
        &self,
        id: u16,
        command: TcpSlaveCommand,
    ) -> Result<(), ChanSendError> {
        match self.clients.get(&id) {
            Some(chan) => chan
                .send(command)
                .map_err(|_| ChanSendError::ChannelIsClosed),

            None => Err(ChanSendError::IdDoesNotExists),
        }
    }

    pub fn new(
        address: SocketAddr,
        creator: SocketAddr,
        pool: TcpIdPool,
    ) -> (Self, ShutdownListener) {
        let (send_chan, recv_chan) = unbounded_channel();
        let (trigger, listener) = shutdown_token();
        (
            Self {
                address,
                creator,
                pool: Arc::new(pool),
                clients: Default::default(),
                send_chan,
                recv_chan,
                _shutdown_token: trigger,
            },
            listener,
        )
    }
}

impl Drop for TcpProxyServer {
    fn drop(&mut self) {
        use owo_colors::OwoColorize;

        tracing::info!(
            "Shut down {}'s {} server",
            self.creator.bold(),
            self.address.bold()
        );
    }
}
