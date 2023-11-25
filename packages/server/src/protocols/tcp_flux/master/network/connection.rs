use std::{
    net::SocketAddr,
    sync::Arc,
};

use flux_common::Rights;
use tokio::sync::mpsc;

use crate::{
    config::root::Config,
    error::NonCriticalError,
    protocols::tcp_flux::{
        error::{
            TcpFluxError,
            TcpFluxResult,
        },
        events::master::MasterEvent,
    },
    user::User,
};

struct MasterChannel {
    tx: mpsc::UnboundedSender<MasterEvent>,
    rx: mpsc::UnboundedReceiver<MasterEvent>,
}

pub struct ConnectionState<'cfg> {
    pub user: User,
    pub(super) config: &'cfg Arc<Config>,

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
    pub fn new(config: &'cfg Arc<Config>, address: SocketAddr) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            user: User::new(Rights::empty(), address),

            channel: MasterChannel { tx, rx },
            config,
        }
    }
}
