use std::sync::Arc;

use cfg_if::cfg_if;
use idpool::flat::FlatIdPool;
use tokio::sync::{
    mpsc,
    Mutex,
};

use super::commands::master::MasterCommand;
use crate::{
    decl::{
        permit_issuers,
        unchecked_unwraps,
    },
    utils::shutdown_token::RaiiShutdown,
};

pub type Pool = Arc<Mutex<FlatIdPool<u16>>>;

cfg_if! {
    if #[cfg(feature = "tcp")] {
        use crate::servers::tcp::TcpServer;
        use super::commands::master::TcpPermit;
    }
}

pub struct Proxy {
    pub pool: Pool,
    pub tx: mpsc::UnboundedSender<MasterCommand>,
    pub rx: mpsc::UnboundedReceiver<MasterCommand>,

    pub data: ProxyData,
    pub _shutdown_sender: RaiiShutdown,
}

pub enum ProxyData {
    #[cfg(feature = "tcp")]
    Tcp(TcpServer),
}

impl Proxy {
    permit_issuers!(tcp);
}

unchecked_unwraps! {
    ProxyData =>
        Tcp: TcpServer
}
