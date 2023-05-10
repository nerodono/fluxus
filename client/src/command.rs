use tokio::sync::OwnedSemaphorePermit;

#[derive(Debug)]
pub struct IdentifiedCommand {
    pub permit: OwnedSemaphorePermit,
    pub id: u16,
    pub command: Command,
}

#[derive(Debug, Clone)]
pub enum Command {
    Forward { buffer: Vec<u8> },
    Disconnect,
}

impl Command {
    pub const fn identified_by(
        self,
        permit: OwnedSemaphorePermit,
        by: u16,
    ) -> IdentifiedCommand {
        IdentifiedCommand {
            permit,
            id: by,
            command: self,
        }
    }
}
