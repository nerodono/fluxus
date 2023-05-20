use std::{
    net::SocketAddr,
    sync::Arc,
};

use rustc_hash::FxHashMap;
use tokio::sync::RwLock;

use super::server::Server;

type InnerMap = FxHashMap<SocketAddr, Arc<Server>>;

#[derive(Clone)]
pub struct ServerMap {
    map: Arc<RwLock<InnerMap>>,
}

impl ServerMap {
    pub async fn map_address(&self, addr: SocketAddr, server: Arc<Server>) {
        let mut map = self.map.write().await;
        map.insert(addr, server);
    }

    pub async fn unmap_address(&self, addr: SocketAddr) {
        let mut map = self.map.write().await;
        map.remove(&addr);
    }
}

impl ServerMap {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ServerMap {
    fn default() -> Self {
        Self {
            map: Arc::new(RwLock::const_new(
                FxHashMap::with_capacity_and_hasher(64, Default::default()),
            )),
        }
    }
}
