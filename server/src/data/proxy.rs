use tokio::sync::mpsc;

use super::{
    commands::base::{
        MasterCommand,
        TcpPermit,
    },
    id_pool::IdPoolImpl,
    servers::tcp::TcpServer,
};
use crate::{
    decl::define_unchecked_mut_unwraps,
    utils::shutdown_token::{
        shutdown_token,
        ShutdownPermit,
        ShutdownToken,
    },
};

pub enum ProxyData {
    Tcp(TcpServer),
}

pub struct ServingProxy {
    send_chan: mpsc::UnboundedSender<MasterCommand>,
    pub recv_chan: mpsc::UnboundedReceiver<MasterCommand>,
    pub data: ProxyData,
    pub pool: IdPoolImpl,

    _permit: ShutdownPermit,
}

impl ServingProxy {
    pub fn new(pool: IdPoolImpl, data: ProxyData) -> (Self, ShutdownToken) {
        let (send_chan, recv_chan) = mpsc::unbounded_channel();
        let (token, permit) = shutdown_token();
        (
            Self {
                send_chan,
                recv_chan,
                data,
                _permit: permit,
                pool,
            },
            token,
        )
    }
}

impl ServingProxy {
    pub fn issue_tcp_permit(&self) -> Option<TcpPermit> {
        match self.data {
            ProxyData::Tcp(..) => {
                Some(unsafe { TcpPermit::new(self.send_chan.clone()) })
            }
        }
    }
}

define_unchecked_mut_unwraps!(ProxyData::[
    Tcp: TcpServer
]);
