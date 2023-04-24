use std::collections::{
    hash_map::Entry,
    HashMap,
};

use tokio::sync::RwLock;

use crate::{
    data::{
        commands::base::HttpPermit,
        id_pool::IdPoolImpl,
    },
    error::HttpEndpointCreationError,
};

pub struct HttpEndpoint {
    pool: IdPoolImpl,
    permit: HttpPermit,
}

#[derive(Default)]
pub struct HttpStorage {
    endpoints: RwLock<HashMap<Vec<u8>, HttpEndpoint>>,
}

impl HttpEndpoint {}

impl HttpStorage {
    pub const fn raw_endpoints(
        &self,
    ) -> &RwLock<HashMap<Vec<u8>, HttpEndpoint>> {
        &self.endpoints
    }

    pub async fn unbind_endpoint(&self, endpoint: &[u8]) {
        self.raw_endpoints()
            .write()
            .await
            .remove(endpoint);
    }

    #[allow(clippy::significant_drop_tightening)]
    pub async fn try_bind_endpoint(
        &self,
        domain_or_path: Vec<u8>,
        pool: IdPoolImpl,
        permit: &HttpPermit,
    ) -> Result<(), HttpEndpointCreationError> {
        let mut endpoints = self.raw_endpoints().write().await;
        let entry = endpoints.entry(domain_or_path);
        match entry {
            Entry::Occupied(..) => Err(HttpEndpointCreationError::Occupied),
            Entry::Vacant(vacant) => {
                vacant.insert(HttpEndpoint {
                    pool,
                    permit: permit.clone(),
                });
                Ok(())
            }
        }
    }

    pub fn new() -> Self {
        Self {
            endpoints: RwLock::new(HashMap::default()),
        }
    }
}
