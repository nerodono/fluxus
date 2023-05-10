use std::sync::Arc;

use cfg_if::cfg_if;
use idpool::flat::FlatIdPool;
use tokio::sync::{
    mpsc,
    Mutex,
};

use super::commands::master::PermittedMasterCommand;
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

cfg_if! {
    if #[cfg(feature = "http")] {
        use crate::servers::http::HttpServer;
        use super::commands::master::HttpPermit;
    }
}

pub struct Proxy {
    pub pool: Pool,
    pub tx: mpsc::Sender<PermittedMasterCommand>,
    pub rx: mpsc::Receiver<PermittedMasterCommand>,

    pub data: ProxyData,
    pub _shutdown_sender: RaiiShutdown,
    pub max_send: u32,
}

pub enum ProxyData {
    #[cfg(feature = "tcp")]
    Tcp(TcpServer),

    #[cfg(feature = "http")]
    Http(HttpServer),
}

impl Proxy {
    permit_issuers!(tcp, http);
}

unchecked_unwraps! {
    ProxyData =>
        Tcp: TcpServer,
        Http: HttpServer
}
