use tcp_flux::connection::{
    master::writer::server::MasterServerWriter,
    traits::RawWrite,
};

use crate::{
    error::CriticalError,
    protocols::tcp_flux::{
        error::{
            TcpFluxError,
            TcpFluxResult,
        },
        events::master::MasterEvent,
        master::network::connection::ConnectionState,
    },
};

// TODO: refactor it (router should only route, not handle
// events)
pub async fn route_event<W>(
    event: MasterEvent,
    writer: &mut MasterServerWriter<W>,
    state: &mut ConnectionState<'_>,
) -> TcpFluxResult<()>
where
    W: RawWrite,
{
    match event {
        MasterEvent::Connected { handshake } => {
            writer.write_connected().await?;
            if state
                .queues
                .tcp
                .push(state.user.address, handshake)
                .is_err()
            {
                return Err(TcpFluxError::Critical(CriticalError::ServerWasShut));
            }
        }

        MasterEvent::ShutdownServer => {
            return Err(TcpFluxError::Critical(CriticalError::ServerWasShut));
        }
    }
    Ok(())
}
