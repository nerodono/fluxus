use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

use tokio::sync::mpsc::UnboundedReceiver;

use crate::data::commands::base::MasterCommand;

pub enum RecvFuture<'a> {
    AlwaysPending,
    Some(&'a mut UnboundedReceiver<MasterCommand>),
}

impl<'a> Future for RecvFuture<'a> {
    type Output = Option<MasterCommand>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        match self.get_mut() {
            Self::AlwaysPending => Poll::Pending,
            Self::Some(chan) => chan.poll_recv(cx),
        }
    }
}
