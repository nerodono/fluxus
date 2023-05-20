use std::net::SocketAddr;

use flux_compression::polymorphic::PolyDctx;
use tokio::sync::{
    mpsc,
    Mutex,
};

use crate::communication::{
    flow::FlowCommand,
    slave::SlaveCommand,
};

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("backlog for socket is full")]
pub struct BacklogIsFull;

pub struct QueuedConnection {
    pub slave_tx: mpsc::Sender<SlaveCommand>,
    pub flow_rx: mpsc::Receiver<FlowCommand>,
    pub dctx: Option<PolyDctx>,
}

struct ConnectionQueue {
    vec: Vec<QueuedConnection>,
    backlog: usize,
}

pub struct Server {
    queue: Mutex<ConnectionQueue>,
    creator: SocketAddr,
}

impl Server {
    pub const fn new(creator: SocketAddr, backlog: usize) -> Self {
        Self {
            queue: Mutex::const_new(ConnectionQueue {
                vec: Vec::new(),
                backlog,
            }),
            creator,
        }
    }
}

impl Server {
    pub async fn dequeue_single(&self) -> Option<QueuedConnection> {
        let mut queue = self.queue.lock().await;
        queue.vec.pop()
    }

    pub async fn enqueue_single(
        &self,
        connection: QueuedConnection,
    ) -> Result<(), BacklogIsFull> {
        let mut queue = self.queue.lock().await;
        if queue.vec.len().saturating_add(1) > queue.backlog {
            Err(BacklogIsFull)
        } else {
            queue.vec.push(connection);
            Ok(())
        }
    }

    pub const fn creator(&self) -> SocketAddr {
        self.creator
    }
}
