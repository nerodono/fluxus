use hyper::{
    http::HeaderValue,
    Method,
};

use super::base::HttpPermit;
use crate::{
    data::id_pool::IdPoolImpl,
    error::HttpEndpointCreationError,
};

pub enum GlobalHttpCommand {
    Bind {
        to: Option<String>,
        permit: HttpPermit,
        pool: IdPoolImpl,
    },

    Unbind {
        domain_or_path: String,
    },
}

pub enum HttpMasterCommand {
    Connected {
        id: u16,
    },
    Request {
        id: u16,
        method: Method,
        headers: Vec<(String, HeaderValue)>,
    },

    FailedToBindEndpoint {
        error: HttpEndpointCreationError,
    },
    BoundEndpoint {
        on: Option<String>,
    },
}
pub enum HttpSlaveCommand {}
