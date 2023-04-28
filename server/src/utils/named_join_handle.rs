use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

use tokio::task::{
    JoinError,
    JoinHandle,
};

pub struct NamedJoinHandle<T> {
    pub name: &'static str,
    pub handle: JoinHandle<T>,
}

impl<T> Future for NamedJoinHandle<T> {
    type Output = (&'static str, Result<T, JoinError>);

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let name = self.name;
        match Pin::new(&mut self.get_mut().handle).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(ready) => Poll::Ready((name, ready)),
        }
    }
}
