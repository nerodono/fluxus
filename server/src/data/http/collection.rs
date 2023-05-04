use std::collections::hash_map::Entry;

use ahash::AHashMap;
use tokio::sync::RwLock;

use super::endpoint::Endpoint;
use crate::data::{
    commands::master::HttpPermit,
    proxy::Pool,
};

pub struct InsertError;
pub struct EraseError;

#[derive(Default)]
pub struct EndpointCollection {
    map: RwLock<AHashMap<Vec<u8>, Endpoint>>,
}

impl EndpointCollection {
    #[allow(clippy::significant_drop_in_scrutinee)]
    pub async fn try_insert_endpoint(
        &self,
        pool: Pool,
        endpoint: Vec<u8>,
        permit: HttpPermit,
    ) -> Result<(), InsertError> {
        let mut map = self.map.write().await;
        match map.entry(endpoint) {
            Entry::Vacant(vacant) => {
                vacant.insert(Endpoint::new(pool, permit));
                Ok(())
            }
            Entry::Occupied(..) => Err(InsertError),
        }
    }

    pub async fn try_erase_entrypoint(
        &self,
        endpoint: &[u8],
    ) -> Result<Endpoint, EraseError> {
        let mut map = self.map.write().await;
        map.remove(endpoint).ok_or(EraseError)
    }

    pub const fn raw_endpoints(
        &self,
    ) -> &RwLock<AHashMap<Vec<u8>, Endpoint>> {
        &self.map
    }

    pub fn new() -> Self {
        Self::default()
    }
}
