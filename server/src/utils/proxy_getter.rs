use crate::{
    data::proxy::Proxy,
    error::{
        NonCriticalError,
        NonCriticalResult,
    },
};

pub fn require_proxy(
    opt: &mut Option<Proxy>,
) -> NonCriticalResult<&mut Proxy> {
    opt.as_mut()
        .ok_or(NonCriticalError::NoServerWasCreated)
}
