use cfg_if::cfg_if;
use tokio::sync::OwnedSemaphorePermit;

use crate::decl::chan_permits;

cfg_if! {
    if #[cfg(feature = "tcp")] {
        use super::tcp::TcpMasterCommand;
    }
}

cfg_if! {
    if #[cfg(feature = "http")] {
        use super::http::IdentifiedHttpMasterCommand;
    }
}

pub struct PermittedMasterCommand {
    pub permit: OwnedSemaphorePermit,
    pub command: MasterCommand,
}

pub enum MasterCommand {
    #[cfg(feature = "tcp")]
    Tcp(TcpMasterCommand),

    #[cfg(feature = "http")]
    Http(IdentifiedHttpMasterCommand),
}

chan_permits! {
    unsafe, PermittedMasterCommand, MasterCommand::[
        Tcp: TcpMasterCommand,
        Http: IdentifiedHttpMasterCommand
    ]
}
