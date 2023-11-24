use std::{
    net::SocketAddr,
    sync::Arc,
};

use tcp_flux::connection::master::{
    reader::common::{
        MasterReader,
        Server,
    },
    writer::server::MasterServerWriter,
};

use super::state::SessionState;
use crate::{
    config::root::Config,
    user::User,
};

pub struct Sides<R, W> {
    pub reader: MasterReader<R, Server>,
    pub writer: MasterServerWriter<W>,
}

pub struct Connection {
    pub(super) state: SessionState,
    pub(super) config: Arc<Config>,
}

impl Connection {
    pub fn new(config: Arc<Config>, address: SocketAddr) -> Self {
        Self {
            state: SessionState {
                user: User::new(address),
            },
            config,
        }
    }
}
