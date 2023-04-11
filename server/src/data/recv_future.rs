use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

use tokio::sync::mpsc::UnboundedReceiver;

pub enum RecvFuture<'a, T> {
    Chan { chan: &'a mut UnboundedReceiver<T> },
    Pending,
}

impl<'a, T> Future for RecvFuture<'a, T> {
    type Output = Option<T>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        match self.get_mut() {
            Self::Chan { chan } => chan.poll_recv(cx),
            Self::Pending => Poll::Pending,
        }
    }
}
