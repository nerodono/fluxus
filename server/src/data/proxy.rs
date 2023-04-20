use tokio::sync::mpsc;

#[cfg(feature = "http")]
use super::commands::base::HttpPermit;
#[cfg(feature = "galaxy")]
use super::commands::base::TcpPermit;
use super::{
    commands::base::MasterCommand,
    servers::{
        http::HttpServer,
        tcp::TcpServer,
    },
};
use crate::{
    decl::{
        define_unchecked_mut_unwraps,
        permit_issuers,
    },
    utils::shutdown_token::{
        shutdown_token,
        ShutdownPermit,
        ShutdownToken,
    },
};

pub enum ProxyData {
    Tcp(TcpServer),
    Http(HttpServer),
}

pub struct ServingProxy {
    send_chan: mpsc::UnboundedSender<MasterCommand>,
    pub recv_chan: mpsc::UnboundedReceiver<MasterCommand>,
    pub data: ProxyData,

    _permit: ShutdownPermit,
}

impl ServingProxy {
    pub fn new(data: ProxyData) -> (Self, ShutdownToken) {
        let (send_chan, recv_chan) = mpsc::unbounded_channel();
        let (token, permit) = shutdown_token();
        (
            Self {
                send_chan,
                recv_chan,
                data,
                _permit: permit,
            },
            token,
        )
    }
}

permit_issuers!(ServingProxy, ProxyData::[
    Tcp("galaxy"),
    Http("http")
]);

define_unchecked_mut_unwraps!(ProxyData::[
    Tcp("galaxy"): TcpServer,
    Http("http"): HttpServer
]);
