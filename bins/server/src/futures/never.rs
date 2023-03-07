use std::{
    convert::Infallible,
    future::Future,
    task::Poll,
};

/// Future that will never stop polling
pub struct NeverCompletes;

impl Future for NeverCompletes {
    type Output = Infallible;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Poll::Pending
    }
}

pub const fn never() -> NeverCompletes {
    NeverCompletes
}
