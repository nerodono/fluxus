use std::sync::Arc;

use galaxy_network::raw::{
    Protocol,
    Rights,
};
use idpool::flat::FlatIdPool;
use tokio::sync::{
    mpsc,
    Mutex,
};

use super::proxy::{
    Pool,
    Proxy,
    ProxyData,
};
use crate::{
    error::{
        NonCriticalError,
        NonCriticalResult,
    },
    utils::{
        recv_future::RecvFuture,
        shutdown_token::{
            shutdown_token,
            ShutdownToken,
        },
    },
};

pub struct User {
    pub proxy: Option<Proxy>,
    pub rights: Rights,
}

impl User {
    pub fn recv_command(&mut self) -> RecvFuture<'_> {
        self.proxy
            .as_mut()
            .map_or(RecvFuture::Pending, |p| RecvFuture::Channel(&mut p.rx))
    }
}

impl User {
    #[must_use = "Shutdown token and id pool should be immediately consumed \
                  by the slave"]
    pub fn replace_proxy<P>(
        &mut self,
        data: ProxyData,
        allocate_channel: usize,
        max_per_permit: u32,
        issuer: impl FnOnce(&Proxy) -> Option<P>,
    ) -> (P, ShutdownToken, Pool) {
        let (tx, rx) = mpsc::channel(allocate_channel);
        let (token, sender) = shutdown_token();
        let pool = Arc::new(Mutex::new(FlatIdPool::new(0)));
        let proxy = Proxy {
            pool: Arc::clone(&pool),
            tx,
            rx,
            data,
            max_send: max_per_permit,
            _shutdown_sender: sender.into(),
        };
        let issued_permit = issuer(&proxy).expect("This should not happen");
        self.proxy = Some(proxy);

        (issued_permit, token, pool)
    }

    pub const fn new(rights: Rights) -> Self {
        Self {
            rights,
            proxy: None,
        }
    }
}

// Helpers
impl User {
    pub const fn select_port(
        &self,
        port: u16,
        protocol: Protocol,
    ) -> NonCriticalResult<u16> {
        Ok(match port {
            0 => 0,
            n if self.rights.can_select_port(protocol) => n,
            _ => {
                return Err(NonCriticalError::NoAccessToSelectPort(
                    port, protocol,
                ))
            }
        })
    }
}
