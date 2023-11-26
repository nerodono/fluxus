use cfg_if::cfg_if;

use super::connection_queue::ConnectionQueue;

cfg_if! {
    if #[cfg(feature = "tcp")] {
        use crate::protocols::tcp_flux::events::flow::FlowHandshake;
    }
}

#[derive(Default, Clone)]
pub struct Queues {
    #[cfg(feature = "tcp")]
    pub tcp: ConnectionQueue<FlowHandshake>,
}
