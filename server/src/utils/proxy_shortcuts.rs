use std::io;

use galaxy_network::{
    error::ReadError,
    raw::ErrorCode,
    reader::ReadResult,
    writer::{
        GalaxyWriter,
        Write,
    },
};

use super::compiler::cold_fn;
use crate::{
    data::proxy::ServingProxy,
    error::SendCommandError,
};

pub async fn treat_send_result<C>(
    writer: &mut GalaxyWriter<impl Write, C>,
    result: Result<(), SendCommandError>,
) -> io::Result<()> {
    if result.is_err() {
        cold_fn();
        writer
            .server()
            .write_error(ErrorCode::ClientDoesNotExists)
            .await
    } else {
        Ok(())
    }
}

pub async fn require_proxy<'a, W: Write, C>(
    writer: &mut GalaxyWriter<W, C>,
    opt: &'a mut Option<ServingProxy>,
) -> ReadResult<&'a mut ServingProxy> {
    if let Some(ref mut serving) = opt {
        Ok(serving)
    } else {
        cold_fn();
        writer
            .server()
            .write_error(ErrorCode::NoServerWasCreated)
            .await?;
        Err(ReadError::NonCritical)
    }
}
