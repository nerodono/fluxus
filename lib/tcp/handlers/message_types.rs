/// Unit struct used as a marker that master wants proxy
/// server shutdown.
#[derive(Debug, Clone, Copy)]
pub struct ShutdownToken;

/// Message to the master
#[derive(Debug)]
pub enum MasterMessage {
    /// Someone is connected to the slave's server. Assigned
    /// id and the newly created channel is returned
    Connected {
        id: u16,
        tx: flume::Sender<SlaveMessage>,
    },

    /// Data was arrived to the slave's socket.
    Forward { id: u16, data: Vec<u8> },

    /// Slave client was disconnected
    Disconnected { id: u16 },

    /// Server shutted down
    Shutdown,
}

/// Message to the slave
#[derive(Debug)]
pub enum SlaveMessage {
    /// Forward data to the slave's socket
    Forward { data: Vec<u8> },

    /// Disconnect slave's client
    Disconnect,
}
