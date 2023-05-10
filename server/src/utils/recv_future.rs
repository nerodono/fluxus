use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

use tokio::sync::mpsc;

use crate::data::commands::master::PermittedMasterCommand;

pub enum RecvFuture<'a, T = PermittedMasterCommand> {
    Channel(&'a mut mpsc::Receiver<T>),
    Pending,
}

impl<'a, T> Future for RecvFuture<'a, T> {
    type Output = Option<T>;

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
