use tokio::sync::mpsc::UnboundedSender;

pub enum TcpMasterCommand {
    Connect {
        id: u16,
        chan: UnboundedSender<TcpSlaveCommand>,
    },
    Forward {
        id: u16,
        buffer: Vec<u8>,
    },
    Disconnect {
        id: u16,
    },

    Stopped,
}

pub enum TcpSlaveCommand {
    Forward { buffer: Vec<u8> },
    Disconnect,
}
