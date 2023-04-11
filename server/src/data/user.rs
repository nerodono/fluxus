use galaxy_network::raw::Rights;

use super::server::tcp::TcpProxyServer;

pub enum Proxy {
    Tcp(TcpProxyServer),
}

pub struct User {
    pub proxy: Option<Proxy>,
    pub rights: Rights,
}

impl User {
    pub const fn new(rights: Rights) -> Self {
        Self {
            rights,
            proxy: None,
        }
    }
}
