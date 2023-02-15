use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

use super::handlers::message::MasterMessage;

pub enum MasterRecvFuture<Fut> {
    Poll(Fut),
    InfinitePolling,
}

pub trait MasterRecvBoundExt {
    type Out;
}

impl<T> MasterRecvBoundExt for MasterRecvFuture<T> {
    type Out = Result<MasterMessage, flume::RecvError>;
}

impl<Fut> Future for MasterRecvFuture<Fut>
where
    Fut: Future<Output = <Self as MasterRecvBoundExt>::Out> + Unpin,
{
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.get_mut() {
            MasterRecvFuture::InfinitePolling => Poll::Pending,
            MasterRecvFuture::Poll(fut) => Pin::new(fut).poll(cx),
        }
    }
}
