use std::future::Future;

use galaxy_network::raw::Rights;

use super::{
    command::MasterCommand,
    recv::RecvFuture,
    tcp_server::TcpProxyServer,
};

pub struct User {
    pub tcp_proxy: Option<TcpProxyServer>,
    pub rights: Rights,
}

impl User {
    #[inline(always)]
    pub fn recv_command(
        &mut self,
    ) -> RecvFuture<impl Future<Output = Option<MasterCommand>> + '_> {
        if let Some(ref mut proxy) = self.tcp_proxy {
            RecvFuture::Custom(proxy.recv_chan.recv())
        } else {
            RecvFuture::Pending
        }
    }

    pub const fn new(rights: Rights) -> Self {
        Self {
            rights,
            tcp_proxy: None,
        }
    }
}
