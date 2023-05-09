use tokio::sync::mpsc;

use super::master::HttpPermit;
use crate::data::proxy::Pool;

pub enum HttpMasterCommand {
    Connected {
        chan: mpsc::UnboundedSender<HttpSlaveCommand>,
        immediate_forward: Vec<u8>,
    },
    Forward {
        buffer: Vec<u8>,
    },

    Disconnected,

    // FIXME: Unidentified
    Bound {
        on: Option<Vec<u8>>,
    },
    FailedToBind,
}

pub struct IdentifiedHttpMasterCommand {
    pub id: u16,
    pub command: HttpMasterCommand,
}

#[derive(Debug)]
pub enum HttpSlaveCommand {
    Forward { buf: Vec<u8> },
    Disconnect,
}

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
