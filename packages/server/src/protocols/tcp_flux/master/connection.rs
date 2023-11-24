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

pub struct ConnectionState<'a> {
    pub(super) user: User,
    pub(super) config: &'a Arc<Config>,
}

impl<'a> ConnectionState<'a> {
    pub fn new(config: &'a Arc<Config>, address: SocketAddr) -> Self {
        Self {
            user: User::new(address),
            config,
        }
    }
}
