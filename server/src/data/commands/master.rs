use cfg_if::cfg_if;

use crate::decl::chan_permits;

cfg_if! {
    if #[cfg(feature = "tcp")] {
        use super::tcp::TcpMasterCommand;
    }
}

pub enum MasterCommand {
    #[cfg(feature = "tcp")]
    Tcp(TcpMasterCommand),
}

chan_permits! {
    MasterCommand::[
        Tcp: TcpMasterCommand
    ]
}
