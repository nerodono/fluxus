use std::sync::Arc;

use mid_idpool::flat::FlatIdPool;
use tokio::sync::Mutex;

use super::handlers::message_types::MasterMessage;

pub struct MasterStateView {
    pub pool: Arc<Mutex<FlatIdPool>>,
    pub master: flume::Sender<MasterMessage>,
}
