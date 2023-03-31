use flume::{
    Receiver,
    Sender,
};
use rustc_hash::FxHashMap;

use super::command::{
    MasterCommand,
    SlaveCommand,
};

pub struct TcpProxyServer {
    send_chan: Sender<MasterCommand>,
    pub(crate) recv_chan: Receiver<MasterCommand>,

    map: FxHashMap<u16, Sender<SlaveCommand>>,
}
