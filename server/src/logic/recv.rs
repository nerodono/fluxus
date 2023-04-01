use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

use tokio::sync::mpsc::UnboundedReceiver;

use super::command::MasterCommand;

pub enum RecvFuture<'a> {
    Custom {
        chan: &'a mut UnboundedReceiver<MasterCommand>,
    },
    Pending,
}

impl<'a> Future for RecvFuture<'a> {
    type Output = Option<MasterCommand>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        match self.get_mut() {
            Self::Custom { chan } => chan.poll_recv(cx),
            Self::Pending => Poll::Pending,
        }
    }
}
