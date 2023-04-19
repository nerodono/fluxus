use std::future::Future;

use galaxy_network::raw::Rights;

use super::{
    commands::base::MasterCommand,
    proxy::ServingProxy,
};
use crate::utils::recv_future::RecvFuture;

pub struct User {
    pub rights: Rights,
    pub proxy: Option<ServingProxy>,
}

impl User {
    pub fn recv_command(
        &mut self,
    ) -> impl Future<Output = Option<MasterCommand>> + '_ {
        match self.proxy {
            Some(ref mut proxy) => RecvFuture::Some(&mut proxy.recv_chan),
            None => RecvFuture::AlwaysPending,
        }
    }

    pub const fn new(rights: Rights) -> Self {
        Self {
            rights,
            proxy: None,
        }
    }
}
