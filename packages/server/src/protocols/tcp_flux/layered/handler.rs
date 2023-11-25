use std::future::Future;

use crate::protocols::tcp_flux::error::TcpFluxResult;

pub struct Truthy;
pub struct Combined<F, N = Truthy> {
    pub lhs: F,
    pub next: N,
}

pub struct Filtered<F, H> {
    pub filter: F,
    pub handler: H,
}

pub struct Handler<F>(pub F);

pub trait HandlerF<P>: Send + Sized {
    fn run(self, payload: P) -> impl Future<Output = TcpFluxResult<()>> + Send;
}

pub trait FilteredExt<P>: Sized {
    fn filter<F>(self, filter: F) -> Filtered<F, Self>;
}

pub trait CombineExt<H, T>: Sized {
    fn combined(self) -> Combined<H, T>;
}

pub trait Filter<P>: Send {
    fn check(self, payload: &P) -> TcpFluxResult<()>;
}

impl<P, H> FilteredExt<P> for H
where
    H: HandlerF<P>,
{
    fn filter<F>(self, filter: F) -> Filtered<F, Self> {
        Filtered {
            filter,
            handler: self,
        }
    }
}

impl<A, B> CombineExt<A, B> for (A, B) {
    fn combined(self) -> Combined<A, B> {
        Combined::new(self.0, self.1)
    }
}

impl<A, B, C> CombineExt<A, Combined<B, C>> for (A, B, C) {
    fn combined(self) -> Combined<A, Combined<B, C>> {
        Combined::new(self.0, Combined::new(self.1, self.2))
    }
}

impl<F, N> Combined<F, N> {
    pub const fn new(lhs: F, next: N) -> Self {
        Self { lhs, next }
    }
}

impl<F, N, P> Filter<P> for Combined<F, N>
where
    F: Filter<P>,
    N: Filter<P>,
{
    fn check(self, payload: &P) -> TcpFluxResult<()> {
        (self.lhs)
            .check(payload)
            .and_then(|()| self.next.check(payload))
    }
}

impl<P> Filter<P> for Truthy {
    fn check(self, _: &P) -> TcpFluxResult<()> {
        Ok(())
    }
}

impl<P, F, H> HandlerF<P> for Filtered<F, H>
where
    P: Send,
    F: Filter<P>,
    H: HandlerF<P>,
{
    fn run(self, payload: P) -> impl Future<Output = TcpFluxResult<()>> + Send {
        async move {
            if let Err(e) = self.filter.check(&payload) {
                Err(e)
            } else {
                self.handler.run(payload).await
            }
        }
    }
}

impl<P, F, Fut> HandlerF<P> for Handler<F>
where
    P: Send,
    F: Send + FnOnce(P) -> Fut,
    Fut: Future<Output = TcpFluxResult<()>> + Send,
{
    fn run(self, payload: P) -> impl Future<Output = TcpFluxResult<()>> + Send {
        self.0(payload)
    }
}
