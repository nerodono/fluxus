use std::sync::Arc;

use tokio::sync::{
    mpsc,
    Notify,
};

use super::master::FlowMasterCommand;

#[derive(Debug)]
pub struct FlowHandshake {
    pub notifier: Arc<Notify>,
    pub flow_tx: mpsc::Sender<FlowMasterCommand>,
    pub master_rx: mpsc::Receiver<FlowEvent>,
}

#[derive(Debug)]
pub enum FlowEvent {
    Wrote { buf: Vec<u8> },
    Closed,
}
