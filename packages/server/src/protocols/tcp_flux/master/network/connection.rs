use std::{
    net::SocketAddr,
    sync::Arc,
};

use flux_common::Rights;
use tokio::sync::{
    mpsc,
    Notify,
};

use crate::{
    config::root::Config,
    error::{
        CriticalError,
        NonCriticalError,
    },
    protocols::tcp_flux::{
        error::{
            TcpFluxError,
            TcpFluxResult,
        },
        events::master::MasterEvent,
    },
    proxies::{
        connection_queue::QueueAlreadyExists,
        queues::Queues,
    },
    user::User,
};

struct MasterChannel {
    tx: mpsc::UnboundedSender<MasterEvent>,
    rx: mpsc::UnboundedReceiver<MasterEvent>,
}

pub struct ConnectionState<'cfg> {
    pub user: User,
    pub queues: &'cfg Queues,
    pub(super) config: &'cfg Arc<Config>,

    shutdown_token: Option<Arc<Notify>>,
    channel: MasterChannel,
}

impl<'cfg> ConnectionState<'cfg> {
    pub const fn require_rights(&self, rights: Rights) -> TcpFluxResult<()> {
        if self.user.rights.contains(rights) {
            Ok(())
        } else {
            Err(TcpFluxError::NonCritical(NonCriticalError::AccessDenied))
        }
    }
}

impl<'cfg> ConnectionState<'cfg> {
    pub fn event_rx(&mut self) -> &mut mpsc::UnboundedReceiver<MasterEvent> {
        &mut self.channel.rx
    }

    pub fn event_tx(&self) -> mpsc::UnboundedSender<MasterEvent> {
        self.channel.tx.clone()
    }
}

impl<'cfg> ConnectionState<'cfg> {
    pub fn create_server(
        &mut self,
        creator: impl FnOnce(SocketAddr, &Queues) -> Result<(), QueueAlreadyExists>,
    ) -> TcpFluxResult<Arc<Notify>> {
        match creator(self.user.address, self.queues) {
            Ok(()) => {
                let token = Arc::new(Notify::new());
                self.shutdown_token = Some(Arc::clone(&token));
                Ok(token)
            }

            Err(_) => Err(TcpFluxError::Critical(CriticalError::FailedToBind)),
        }
    }
}

impl<'cfg> ConnectionState<'cfg> {
    pub fn new(
        queues: &'cfg Queues,
        config: &'cfg Arc<Config>,
        address: SocketAddr,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            queues,
            shutdown_token: None,
            user: User::new(Rights::empty(), address),

            channel: MasterChannel { tx, rx },
            config,
        }
    }
}

impl<'cfg> Drop for ConnectionState<'cfg> {
    fn drop(&mut self) {
        if let Some(ref mut token) = self.shutdown_token {
            token.notify_waiters();

            // Cleanup queue
            _ = self.queues.tcp.drop_queue(&self.user.address);
        }
    }
}
