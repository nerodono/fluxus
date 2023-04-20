use tokio::sync::mpsc::UnboundedSender;

use super::{
    http::HttpMasterCommand,
    tcp::TcpMasterCommand,
};
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
