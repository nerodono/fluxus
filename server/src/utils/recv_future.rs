use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

use tokio::sync::mpsc;

use crate::data::commands::master::MasterCommand;

pub enum RecvFuture<'a> {
    Channel(&'a mut mpsc::UnboundedReceiver<MasterCommand>),
    Pending,
}

impl<'a> Future for RecvFuture<'a> {
    type Output = Option<MasterCommand>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        match self.get_mut() {
            Self::Channel(chan) => Pin::new(chan).poll_recv(cx),
            Self::Pending => Poll::Pending,
        }
    }
}
