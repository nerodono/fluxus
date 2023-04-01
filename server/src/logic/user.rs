use galaxy_network::raw::{
    Protocol,
    Rights,
};

use super::{
    recv::RecvFuture,
    tcp_server::TcpProxyServer,
};

pub struct User {
    pub tcp_proxy: Option<TcpProxyServer>,
    pub rights: Rights,
}

impl User {
    pub const fn can_create_server(&self, protocol: Protocol) -> bool {
        match protocol {
            Protocol::Http => {
                self.rights.intersects(Rights::CAN_CREATE_HTTP)
            }
            Protocol::Tcp => {
                self.rights.intersects(Rights::CAN_CREATE_TCP)
            }
            Protocol::Udp => {
                self.rights.intersects(Rights::CAN_CREATE_UDP)
            }
        }
    }

    pub const fn can_select_port(&self, protocol: Protocol) -> bool {
        match protocol {
            Protocol::Http => false,
            Protocol::Tcp => self
                .rights
                .intersects(Rights::CAN_SELECT_TCP_PORT),
            Protocol::Udp => self
                .rights
                .intersects(Rights::CAN_SELECT_UDP_PORT),
        }
    }
}

impl User {
    #[inline(always)]
    pub fn recv_command(&mut self) -> RecvFuture<'_> {
        if let Some(ref mut proxy) = self.tcp_proxy {
            RecvFuture::Custom {
                chan: &mut proxy.recv_chan,
            }
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
