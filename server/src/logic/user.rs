use std::future::Future;

use flume::RecvError;
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
        &self,
    ) -> RecvFuture<
        impl Unpin + Future<Output = Result<MasterCommand, RecvError>> + '_,
    > {
        if let Some(ref proxy) = self.tcp_proxy {
            RecvFuture::Custom(proxy.recv_chan.recv_async())
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
