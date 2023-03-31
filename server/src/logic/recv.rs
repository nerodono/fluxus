use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

use flume::RecvError;

use super::command::MasterCommand;

pub enum RecvFuture<F> {
    Custom(F),
    Pending,
}

impl<F> Future for RecvFuture<F>
where
    F: Unpin + Future<Output = Result<MasterCommand, RecvError>>,
{
    type Output = Result<MasterCommand, RecvError>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        match self.get_mut() {
            Self::Custom(ref mut f) => Pin::new(f).poll(cx),
            Self::Pending => Poll::Pending,
        }
    }
}
