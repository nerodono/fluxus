use crate::{
    data::commands::{
        http::HttpMasterCommand,
        master::HttpPermit,
    },
    error::HttpResult,
};

pub struct Destination {
    pub id: u16,
    pub dest_id: Vec<u8>,
    pub permit: HttpPermit,
}

impl Destination {
    pub fn send(&self, command: HttpMasterCommand) -> HttpResult<()> {
        self.permit
            .send(command.identified(self.id))
            .map_err(Into::into)
    }

    pub const fn new(id: u16, dest_id: Vec<u8>, permit: HttpPermit) -> Self {
        Self {
            id,
            dest_id,
            permit,
        }
    }
}

impl Drop for Destination {
    fn drop(&mut self) {
        _ = self.send(HttpMasterCommand::Disconnected);
    }
}
