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

use crate::{
    config::root::Config,
    user::User,
};

pub struct Sides<R, W> {
    pub reader: MasterReader<R, Server>,
    pub writer: MasterServerWriter<W>,
}

pub struct ConnectionState {
    pub(super) user: User,
    pub(super) config: Arc<Config>,
}

impl ConnectionState {
    pub fn new(config: Arc<Config>, address: SocketAddr) -> Self {
        Self {
            user: User::new(address),
            config,
        }
    }
}
