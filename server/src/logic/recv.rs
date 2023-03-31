use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

use super::command::MasterCommand;

pub enum RecvFuture<F> {
    Custom(F),
    Pending,
}

impl<F> Future for RecvFuture<F>
where
    F: Unpin + Future<Output = Option<MasterCommand>>,
{
    type Output = Option<MasterCommand>;

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
