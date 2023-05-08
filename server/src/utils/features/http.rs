use tokio::sync::mpsc;

use crate::{
    data::commands::http::HttpServerRequest,
    error::{
        NonCriticalError,
        NonCriticalResult,
    },
};

#[derive(Clone)]
pub struct HttpChannel {
    tx: mpsc::UnboundedSender<HttpServerRequest>,
}

impl HttpChannel {
    pub const fn new(tx: mpsc::UnboundedSender<HttpServerRequest>) -> Self {
        Self { tx }
    }
}

impl HttpChannel {
    pub fn send(&self, request: HttpServerRequest) -> NonCriticalResult<()> {
        self.tx
            .send(request)
            .map_err(|_| NonCriticalError::FeatureIsUnavailable("http"))
    }
}
