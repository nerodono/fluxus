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

pub type TcpIdPool = Box<dyn IdPool<Id = u16> + Send>;

pub struct TcpProxyServer {
    send_chan: UnboundedSender<MasterCommand>,
    pub(crate) recv_chan: UnboundedReceiver<MasterCommand>,

    map: FxHashMap<u16, UnboundedSender<SlaveCommand>>,
    pool: Mutex<TcpIdPool>,
}

impl TcpProxyServer {
    pub fn new(pool: TcpIdPool) -> Self {
        let (send_chan, recv_chan) = unbounded_channel();
        Self {
            pool: Mutex::new(pool),
            map: Default::default(),
            send_chan,
            recv_chan,
        }
    }
}
