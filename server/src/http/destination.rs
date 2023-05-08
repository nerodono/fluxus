use tokio::sync::mpsc;

use crate::{
    data::commands::{
        http::{
            HttpMasterCommand,
            HttpSlaveCommand,
        },
        master::HttpPermit,
    },
    error::HttpResult,
    utils::recv_future::RecvFuture,
};

pub struct Destination {
    pub id: u16,
    pub dest_id: Vec<u8>,
    pub permit: HttpPermit,

    pub rx: mpsc::UnboundedReceiver<HttpSlaveCommand>,
    pub discovered: bool,

    notify_disconnected: bool,
}

impl Destination {
    pub fn same_endpoint(&self, endpoint: &[u8]) -> bool {
        self.dest_id == endpoint
    }

    pub fn dont_notify(&mut self) {
        self.notify_disconnected = false;
    }

    pub fn recv_command(
        maybe_this: Option<&mut Self>,
    ) -> RecvFuture<'_, HttpSlaveCommand> {
        match maybe_this {
            Some(this) => RecvFuture::Channel(&mut this.rx),
            None => RecvFuture::Pending,
        }
    }

    pub fn send(&self, command: HttpMasterCommand) -> HttpResult<()> {
        self.permit
            .send(command.identified(self.id))
            .map_err(Into::into)
    }

    pub const fn new(
        id: u16,
        dest_id: Vec<u8>,
        permit: HttpPermit,
        rx: mpsc::UnboundedReceiver<HttpSlaveCommand>,
    ) -> Self {
        Self {
            id,
            dest_id,
            permit,
            rx,
            notify_disconnected: true,
            discovered: true,
        }
    }
}

impl Drop for Destination {
    fn drop(&mut self) {
        if self.notify_disconnected {
            _ = self.send(HttpMasterCommand::Disconnected);
        }
    }
}
