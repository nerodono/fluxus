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

    pub rx: mpsc::Receiver<HttpSlaveCommand>,
    pub discovered: bool,

    notify_disconnected: bool,
}

impl Destination {
    pub async fn send_if_valid(
        this: &Option<Self>,
        command: impl FnOnce() -> HttpMasterCommand,
    ) -> HttpResult<()> {
        if let Some(this) = Self::valid_for_send(this) {
            this.send(command()).await
        } else {
            Ok(())
        }
    }

    pub const fn valid_for_send(this: &Option<Self>) -> Option<&Self> {
        if let Some(
            ref this @ Self {
                discovered: true, ..
            },
        ) = this
        {
            Some(this)
        } else {
            None
        }
    }

    pub fn valid_for_send_mut(this: &mut Option<Self>) -> Option<&mut Self> {
        if let Some(
            ref mut this @ Self {
                discovered: true, ..
            },
        ) = this
        {
            Some(this)
        } else {
            None
        }
    }

    pub fn set_discovered_opt(this: &mut Option<Self>, to: bool) {
        if let Some(ref mut this) = this {
            this.discovered = to;
        }
    }

    pub fn same_endpoint(&self, endpoint: &[u8]) -> bool {
        self.dest_id == endpoint
    }

    pub fn dont_notify(&mut self) {
        self.notify_disconnected = false;
    }

    pub fn recv_command(
        maybe_this: &mut Option<Self>,
    ) -> RecvFuture<'_, HttpSlaveCommand> {
        match maybe_this {
            Some(ref mut this) => RecvFuture::Channel(&mut this.rx),
            None => RecvFuture::Pending,
        }
    }

    pub async fn send(&self, command: HttpMasterCommand) -> HttpResult<()> {
        self.permit
            .send(command.identified(self.id))
            .await
            .map_err(Into::into)
    }

    pub const fn new(
        id: u16,
        dest_id: Vec<u8>,
        permit: HttpPermit,
        rx: mpsc::Receiver<HttpSlaveCommand>,
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
            let permit = self.permit.clone();
            let id = self.id;
            tokio::spawn(async move {
                _ = permit
                    .send(HttpMasterCommand::Disconnected.identified(id))
                    .await;
            });
        }
    }
}
