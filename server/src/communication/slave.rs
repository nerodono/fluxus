#[derive(Debug, Clone)]
pub enum SlaveCommand {
    Forward { buffer: Vec<u8> },
    Disconnect,
}
