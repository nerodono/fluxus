use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub enum TcpSlaveCommand {
    Forward { buffer: Vec<u8> },
    Disconnect,
}

#[derive(Debug)]
pub enum TcpMasterCommand {
    Forward {
        id: u16,
        buffer: Vec<u8>,
    },
    Connected {
        id: u16,
        channel: UnboundedSender<TcpSlaveCommand>,
    },
    Disconnected {
        id: u16,
    },

    Stopped,
}
