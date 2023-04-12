use tokio::sync::mpsc;

use super::{
    command::erased::ErasedCommand,
    idpool::IdPool,
    server::tcp::TcpProxyServer,
};

pub struct Proxy {
    pub send_chan: mpsc::UnboundedSender<ErasedCommand>,
    pub recv_chan: mpsc::UnboundedReceiver<ErasedCommand>,
    pub id_pool: IdPool,

    pub concrete: Option<ConcreteProxy>,
}

pub enum ConcreteProxy {
    Tcp(TcpProxyServer),
}

impl Proxy {
    pub fn new(id_pool: IdPool) -> Self {
        let (send_chan, recv_chan) = mpsc::unbounded_channel();
        Self {
            send_chan,
            recv_chan,
            id_pool,
            concrete: None,
        }
    }
}
