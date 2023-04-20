use std::collections::{
    hash_map::Entry,
    HashMap,
};

use hyper::{
    body::Bytes,
    http::HeaderValue,
};
use idpool::interface::IdPool;
use tokio::sync::RwLock;

use crate::{
    data::{
        commands::{
            base::HttpPermit,
            http::HttpMasterCommand,
        },
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
    endpoints: RwLock<HashMap<String, HttpEndpoint>>,
}

impl HttpEndpoint {
    pub async fn notify_connection(&self) -> Option<u16> {
        self.pool.lock().await.request().and_then(|id| {
            self.permit
                .send(HttpMasterCommand::Connected { id })
                .ok()
                .map(|()| id)
        })
    }
}

impl HttpStorage {
    pub const fn raw_endpoints(
        &self,
    ) -> &RwLock<HashMap<String, HttpEndpoint>> {
        &self.endpoints
    }

    pub async fn unbind_endpoint(&self, endpoint: &str) {
        self.raw_endpoints()
            .write()
            .await
            .remove(endpoint);
    }

    pub async fn try_bind_endpoint(
        &self,
        domain_or_path: String,
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
