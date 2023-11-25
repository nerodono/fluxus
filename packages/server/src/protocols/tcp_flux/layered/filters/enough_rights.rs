use flux_common::Rights;

use crate::{
    error::NonCriticalError,
    protocols::tcp_flux::{
        error::{
            TcpFluxError,
            TcpFluxResult,
        },
        layered::handler::Filter,
    },
};

pub struct EnoughRightsF<F> {
    pub current: Rights,
    pub required: F,
}

pub struct EnoughRights {
    pub current: Rights,
    pub required: Rights,
}

pub fn enough_rights(current: Rights, required: Rights) -> EnoughRights {
    EnoughRights { current, required }
}

pub fn enough_rights_lazy<P, F>(current: Rights, required: F) -> EnoughRightsF<F>
where
    F: FnOnce(&P) -> Rights,
{
    EnoughRightsF { current, required }
}

impl<P> Filter<P> for EnoughRights {
    fn check(self, _: &P) -> TcpFluxResult<()> {
        if self.current.contains(self.required) {
            Ok(())
        } else {
            Err(TcpFluxError::NonCritical(NonCriticalError::AccessDenied))
        }
    }
}

impl<F, P> Filter<P> for EnoughRightsF<F>
where
    F: Send + for<'lt> FnOnce(&'lt P) -> Rights,
{
    fn check(self, payload: &P) -> TcpFluxResult<()> {
        if self.current.contains((self.required)(payload)) {
            Ok(())
        } else {
            Err(TcpFluxError::NonCritical(NonCriticalError::AccessDenied))
        }
    }
}
