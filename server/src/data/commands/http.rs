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
    FailedToBindEndpoint { error: HttpEndpointCreationError },
    BoundEndpoint { on: Option<String> },
}
pub enum HttpSlaveCommand {}
