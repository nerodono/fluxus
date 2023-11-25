use std::{
    net::SocketAddr,
    sync::Arc,
};

use flux_common::Rights;
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

pub struct ConnectionState<'cfg> {
    pub(super) user: User,
    pub(super) config: &'cfg Arc<Config>,
}

impl<'cfg> ConnectionState<'cfg> {
    pub fn new(config: &'cfg Arc<Config>, address: SocketAddr) -> Self {
        Self {
            user: User::new(Rights::empty(), address),
            config,
        }
    }
}
