use crate::protocols::tcp_flux::{
    error::{
        TcpFluxError,
        TcpFluxResult,
    },
    layered::handler::Filter,
};

pub struct Predicate<F, Ef> {
    pub pred: F,
    pub error: Ef,
}

pub fn predicate<F, Ef>(pred: F, error: Ef) -> Predicate<F, Ef> {
    Predicate { pred, error }
}

impl<P, F, Ef, R> Filter<P> for Predicate<F, Ef>
where
    F: Send + FnOnce() -> bool,
    Ef: Send + FnOnce() -> R,
    R: Into<TcpFluxError>,
{
    fn check(self, _: &P) -> TcpFluxResult<()> {
        if (self.pred)() {
            Ok(())
        } else {
            Err((self.error)().into())
        }
    }
}
