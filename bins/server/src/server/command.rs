use tokio::sync::mpsc;

#[derive(Debug)]
pub enum MasterCommand {
    Connected {
        id: u16,
        tx: mpsc::UnboundedSender<SlaveCommand>,
    },
    Forward {
        id: u16,
        buffer: Vec<u8>,
    },
    Disconnected {
        id: u16,
    },
}

#[derive(Debug)]
pub enum SlaveCommand {
    Forward { buffer: Vec<u8> },
    ForceDisconnect,
}
