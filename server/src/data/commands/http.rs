use super::master::HttpPermit;
use crate::data::proxy::Pool;

pub enum HttpMasterCommand {
    Connected,
    Header { buf: Vec<u8> },
    BodyChunk { buf: Vec<u8> },

    Disconnected,

    // FIXME: Unidentified
    Bound { on: Option<Vec<u8>> },
    FailedToBind,
}

pub struct IdentifiedHttpMasterCommand {
    pub id: u16,
    pub command: HttpMasterCommand,
}

pub enum HttpSlaveCommand {}

pub enum HttpServerRequest {
    Bind {
        endpoint: Option<Vec<u8>>,
        permit: HttpPermit,
        pool: Pool,
    },
    Unbind {
        endpoint: Vec<u8>,
    },
}

impl HttpMasterCommand {
    // FIXME: slightly restructure project to remove this
    pub const fn unidentified(self) -> IdentifiedHttpMasterCommand {
        IdentifiedHttpMasterCommand {
            id: u16::MAX,
            command: self,
        }
    }

    pub const fn identified(self, id: u16) -> IdentifiedHttpMasterCommand {
        IdentifiedHttpMasterCommand { id, command: self }
    }
}
