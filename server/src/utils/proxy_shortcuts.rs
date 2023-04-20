use super::compiler::cold_fn;
use crate::{
    data::proxy::ServingProxy,
    error::{
        NonCriticalError,
        NonCriticalResult,
        SendCommandError,
    },
};

pub const fn treat_send_result(
    result: Result<(), SendCommandError>,
) -> NonCriticalResult<()> {
    if result.is_err() {
        cold_fn();
        Err(NonCriticalError::ClientDoesNotExists)
    } else {
        Ok(())
    }
}

pub fn require_proxy(
    opt: &mut Option<ServingProxy>,
) -> NonCriticalResult<&mut ServingProxy> {
    opt.as_mut().map_or_else(
        || {
            cold_fn();
            Err(NonCriticalError::NoServer)
        },
        Ok,
    )
}
