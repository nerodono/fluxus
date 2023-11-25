use tcp_flux::connection::{
    master::{
        reader::common::{
            MasterReader,
            Server,
        },
        writer::server::MasterServerWriter,
    },
    traits::{
        RawRead,
        RawWrite,
    },
};

use super::{
    event::router::route_event,
    network::{
        connection::ConnectionState,
        router::route_packet,
    },
};
use crate::{
    error::CriticalError,
    protocols::tcp_flux::error::{
        convert_critical,
        convert_non_critical,
        TcpFluxError,
        TcpFluxResult,
    },
};

pub async fn handle_connection<'cfg, R, W>(
    mut reader: MasterReader<R, Server>,
    mut writer: MasterServerWriter<W>,
    mut state: ConnectionState<'cfg>,
) -> TcpFluxResult<()>
where
    R: RawRead,
    W: RawWrite,
{
    loop {
        let result = tokio::select! {
            network_result = reader.next_packet() => {
                let (pkt, reader) = network_result?;
                route_packet(
                    reader,
                    &mut writer,
                    &mut state,
                    pkt
                )
                .await
            }

            event_result = state.event_rx().recv() => {
                route_event(
                    event_result.ok_or(CriticalError::ChannelClosed)?,
                    &mut writer,
                    &mut state
                )
                .await
            }
        };

        if let Err(e) = result {
            match e {
                TcpFluxError::Critical(error) => {
                    tracing::error!("{} critical error: {error}", state.user);
                    return writer
                        .write_error(convert_critical(error))
                        .await
                        .map_err(TcpFluxError::Io);
                }

                TcpFluxError::NonCritical(error) => {
                    tracing::error!("{} non-critical error: {error}", state.user);
                    writer
                        .write_error(convert_non_critical(error))
                        .await?;
                }

                other => return Err(other),
            }
        }
    }
}
