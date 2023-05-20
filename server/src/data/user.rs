use std::{
    fmt::Display,
    mem,
    net::SocketAddr,
};

use flux_tcp::raw::Rights;
use owo_colors::OwoColorize;

use super::server_map::ServerMap;
use crate::{
    config::RightsScope,
    traits::FromConfig,
    utils::shutdown_token::ShutdownWriter,
};

struct CreatedServer {
    address: SocketAddr,
    _writer: ShutdownWriter,
    map: Option<ServerMap>,
}

pub struct User {
    pub rights: Rights,

    server: Option<CreatedServer>,
    address: SocketAddr,
}

impl User {
    pub fn set_server(
        &mut self,
        opt_writer: Option<(ShutdownWriter, ServerMap)>,
    ) {
        self.server = opt_writer.map(|(writer, map)| CreatedServer {
            address: self.address,
            _writer: writer,
            map: Some(map),
        });
    }

    pub const fn address(&self) -> SocketAddr {
        self.address
    }

    pub const fn new(address: SocketAddr, rights: Rights) -> Self {
        Self {
            server: None,
            address,
            rights,
        }
    }
}

impl Drop for CreatedServer {
    fn drop(&mut self) {
        let map = mem::take(&mut self.map).unwrap();
        let addr = self.address;

        tokio::spawn(async move {
            map.unmap_address(addr).await;
        });
    }
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let server_text = if self.server.is_some() {
            "hosted"
        } else {
            "serverless"
        };

        write!(f, "{}:{}", self.address().bold(), server_text.red())
    }
}

impl FromConfig<RightsScope> for Rights {
    fn from_config(entry: &RightsScope) -> Self {
        let mut rights = Self::empty();

        if entry.http.create {
            rights |= Rights::CREATE_HTTP;
        }
        if entry.http.custom_host {
            rights |= Rights::SELECT_HTTP_HOST;
        }

        if entry.tcp.create {
            rights |= Rights::CREATE_TCP;
        }
        if entry.tcp.select_port {
            rights |= Rights::SELECT_TCP_PORT;
        }

        rights
    }
}
