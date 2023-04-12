use std::future::Future;

use tokio::sync::mpsc;

use crate::data::command::tcp::{
    TcpMasterCommand,
    TcpSlaveCommand,
};

pub struct TcpProxyServer {
    pub send_chan: mpsc::UnboundedSender<TcpMasterCommand>,
    pub recv_chan: mpsc::UnboundedReceiver<TcpMasterCommand>,
}
