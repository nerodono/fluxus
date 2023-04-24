use cfg_if::cfg_if;
use tokio::sync::mpsc::UnboundedSender;

cfg_if! {
    if #[cfg(feature = "http")] {
        use super::http::HttpMasterCommand;
    }
}

cfg_if! {
    if #[cfg(feature = "galaxy")] {
        use super::tcp::TcpMasterCommand;
    }
}

use crate::decl::chan_permits;

pub enum MasterCommand {
    #[cfg(feature = "galaxy")]
    Tcp(TcpMasterCommand),

    #[cfg(feature = "http")]
    Http(HttpMasterCommand),
}

chan_permits!(UnboundedSender, MasterCommand::[
    Tcp("galaxy"): TcpMasterCommand,
    Http("http"): HttpMasterCommand,
]);
