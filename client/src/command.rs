#[derive(Debug)]
pub struct IdentifiedCommand {
    pub id: u16,
    pub command: Command,
}

#[derive(Debug, Clone)]
pub enum Command {
    Forward { buffer: Vec<u8> },
    Disconnect,
}

impl Command {
    pub const fn identified_by(self, by: u16) -> IdentifiedCommand {
        IdentifiedCommand {
            id: by,
            command: self,
        }
    }
}
