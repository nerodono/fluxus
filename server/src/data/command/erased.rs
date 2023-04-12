use super::tcp::TcpMasterCommand;
use crate::decl::chan_permits;

pub enum ErasedCommand {
    Tcp(TcpMasterCommand),
}

chan_permits! {
    ErasedCommand::[
        [Tcp, TcpMasterCommand]
    ]
}
