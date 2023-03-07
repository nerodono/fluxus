use galaxy_net::schemas::Permissions;
use tokio::sync::mpsc;

use super::command::MasterCommand;
use crate::futures::never::never;

pub struct TcpServer {
    pub master_tx: mpsc::UnboundedSender<MasterCommand>,
    pub master_rx: mpsc::UnboundedReceiver<MasterCommand>,
}

pub struct Client {
    pub perms: Permissions,
    pub tcp_server: Option<TcpServer>,
}

impl Client {
    /// Pulls command from the Tcp server. Polls forever if
    /// server was not created
    pub async fn pull_tcp_command(
        &mut self,
    ) -> Option<MasterCommand> {
        if let Some(ref mut server) = self.tcp_server {
            server.master_rx.recv().await
        } else {
            // Dirty-funny trick
            match never().await {}
        }
    }
}
