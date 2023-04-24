use super::base::HttpPermit;
use crate::{
    data::id_pool::IdPoolImpl,
    error::HttpEndpointCreationError,
};

pub enum GlobalHttpCommand {
    Bind {
        to: Option<Vec<u8>>,
        permit: HttpPermit,
        pool: IdPoolImpl,
    },

    Unbind {
        domain_or_path: Vec<u8>,
    },
}

pub enum HttpMasterCommand {
    FailedToBindEndpoint { error: HttpEndpointCreationError },
    BoundEndpoint { on: Option<String> },
}
pub enum HttpSlaveCommand {}
