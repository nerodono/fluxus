use tokio::sync::mpsc::UnboundedSender;

use super::tcp::TcpMasterCommand;
use crate::decl::chan_permits;

pub enum MasterCommand {
    Tcp(TcpMasterCommand),
}

chan_permits!(UnboundedSender, MasterCommand::[
    Tcp: TcpMasterCommand
]);
