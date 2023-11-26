use super::flow::FlowHandshake;

#[derive(Debug)]
pub enum FlowMasterCommand {
    Forward { buf: Vec<u8> },
    Close,
}

#[derive(Debug)]
pub enum MasterEvent {
    Connected { handshake: FlowHandshake },
    ShutdownServer,
}
