use std::{
    future::Future,
    mem,
    sync::Arc,
};

use bitflags::bitflags;
use flume::{
    Receiver,
    Sender,
};
use mid_idpool::flat::FlatIdPool;
use rustc_hash::FxHashMap;
use tokio::sync::{
    oneshot,
    Mutex,
};

use super::{
    handlers::message::{
        MasterMessage,
        ShutdownToken,
        SlaveMessage,
    },
    recv_future::{
        MasterRecvBoundExt,
        MasterRecvFuture,
    },
};
use crate::config::base::ProtocolPermissionsCfg;

bitflags! {
    pub struct Permissions: u16 {
        const CREATE_TCP      = 1 << 0;
        const SELECT_TCP_PORT = 1 << 1;

        const CREATE_UDP      = 1 << 2;
        const SELECT_UDP_PORT = 1 << 3;
    }
}

/// Created server state.
pub struct CreatedServer {
    pub rx: Receiver<MasterMessage>,
    pub map: FxHashMap<u16, flume::Sender<SlaveMessage>>,
    pub pool: Arc<Mutex<FlatIdPool>>,

    port: u16,
    token: Option<oneshot::Sender<ShutdownToken>>,
}

/// User state during the connection.
pub struct State {
    pub permissions: Permissions,
    pub server: Option<CreatedServer>,
}

impl State {
    /// Receives master message or returns future that will
    /// be pending infinitely if server is not created.
    pub fn recv_master_message(
        &mut self,
    ) -> MasterRecvFuture<
        impl Future<Output = <MasterRecvFuture<()> as MasterRecvBoundExt>::Out> + '_,
    > {
        match self.server {
            Some(ref mut server) => {
                MasterRecvFuture::Poll(server.rx.recv_async())
            }
            None => MasterRecvFuture::InfinitePolling,
        }
    }

    /// Creates server state on the connection. Returns
    /// shutdown token, master sender and created server
    pub fn create_server(
        &mut self,
        port: u16,
    ) -> (
        oneshot::Receiver<ShutdownToken>,
        Sender<MasterMessage>,
        &mut CreatedServer,
    ) {
        let (tx, rx) = flume::unbounded();
        let (stx, srx) = oneshot::channel();

        self.server = Some(CreatedServer {
            rx,
            map: FxHashMap::with_capacity_and_hasher(4, Default::default()),
            pool: Arc::new(Mutex::new(FlatIdPool::new())),
            token: Some(stx),
            port,
        });

        (srx, tx, self.server.as_mut().unwrap())
    }

    /// Checks whether user created server or not.
    pub fn has_server(&self) -> bool {
        self.server.is_some()
    }

    /// Create state from the permissions configuration
    pub fn new(permissions: &ProtocolPermissionsCfg) -> Self {
        Self {
            permissions: Permissions::from_cfg(permissions),
            server: None,
        }
    }
}

impl Permissions {
    /// Check whether user has supplied permissions
    pub const fn can(self, perms: Permissions) -> bool {
        self.contains(perms)
    }

    /// Create [`Permissions`] from the configuration
    /// struct.
    pub fn from_cfg(permissions: &ProtocolPermissionsCfg) -> Self {
        let mut perms = Permissions::empty();

        if permissions.tcp.create_server {
            perms |= Permissions::CREATE_TCP;
        }
        if permissions.tcp.select_port {
            perms |= Permissions::SELECT_TCP_PORT;
        }

        if permissions.udp.create_server {
            perms |= Permissions::CREATE_UDP;
        }
        if permissions.udp.select_port {
            perms |= Permissions::SELECT_UDP_PORT;
        }

        perms
    }
}

impl CreatedServer {
    pub async fn unlisten_slave(&mut self, id: u16) -> Result<(), ()> {
        if self.map.remove(&id).is_none() {
            return Err(());
        }

        self.pool.lock().await.push_back_unchecked(id);
        Ok(())
    }

    /// Forcibly disconnect slave
    pub async fn force_disconnect(&mut self, id: u16) -> Result<(), ()> {
        self.send_message(id, SlaveMessage::Disconnect)
            .await?;
        self.unlisten_slave(id).await
    }

    /// Send forward message to the slave
    pub fn forward(
        &self,
        id: u16,
        data: Vec<u8>,
    ) -> impl Future<Output = Result<(), ()>> + '_ {
        self.send_message(id, SlaveMessage::Forward { data })
    }

    /// Send message to the slave
    pub async fn send_message(
        &self,
        id: u16,
        message: SlaveMessage,
    ) -> Result<(), ()> {
        if let Some(tx) = self.map.get(&id) {
            tx.send_async(message).await.map_err(|_| ())
        } else {
            Err(())
        }
    }
}

impl Drop for CreatedServer {
    fn drop(&mut self) {
        let chan = mem::take(&mut self.token).unwrap();
        chan.send(ShutdownToken).unwrap_or(());

        tracing::info!(port = self.port, "Destroyed server");
    }
}
