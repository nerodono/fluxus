#[derive(Debug, Clone)]
pub enum FlowCommand {
    Forward { buffer: Vec<u8>, compressed: bool },
    Disconnect,
}
