use tokio::sync::mpsc;

#[derive(Debug)]
pub enum TcpMasterCommand {
    Connected {
        id: u16,
        chan: mpsc::Sender<TcpSlaveCommand>,
    },

    Forward {
        id: u16,
        buffer: Vec<u8>,
    },

    Disconnected {
        id: u16,
    },

    Stopped,
}

#[derive(Debug)]
pub enum TcpSlaveCommand {
    Forward { buffer: Vec<u8> },
    Disconnect,
}
