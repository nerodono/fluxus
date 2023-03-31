use flume::Sender;

#[derive(Debug)]
pub enum SlaveCommand {
    Forward { buffer: Vec<u8> },
    Disconnect,
}

#[derive(Debug)]
pub enum MasterCommand {
    Forward {
        id: u16,
        buffer: Vec<u8>,
    },
    Connected {
        id: u16,
        channel: Sender<SlaveCommand>,
    },
    Disconneceted {
        id: u16,
    },

    Stopped,
}