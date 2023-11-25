use tcp_flux::{
    connection::traits::{
        RawRead,
        RawWrite,
    },
    types::pkt_base::PktType,
};

use super::connection::{
    ConnectionState,
    Sides,
};
use crate::{
    error::CriticalError,
    protocols::tcp_flux::{
        error::{
            convert_critical,
            convert_non_critical,
            TcpFluxError,
            TcpFluxResult,
        },
        master::atom::Atom,
    },
};

pub async fn route_packets<R, W>(
    mut net: Sides<R, W>,
    mut state: ConnectionState<'_>,
) -> TcpFluxResult<()>
where
    R: RawRead,
    W: RawWrite,
{
    use PktType as P;
    loop {
        let (pkt, reader) = net.reader.next_packet().await?;
        let mut atom = Atom::new(&mut state, reader, &mut net.writer, pkt.flags);

        let result = match pkt.type_ {
            P::Authenticate => atom.authenticate().await,
            P::ReqInfo => atom.req_info().await,
            P::Disconnect => atom.disconnect().await,
            P::CreateHttp => atom.create_http().await,
            P::CreateTcp => atom.create_tcp().await,

            P::Connected | P::Error | P::UpdateRights => {
                Err(TcpFluxError::Critical(CriticalError::UnexpectedPacket))
            }
        };

        if let Err(e) = result {
            match e {
                TcpFluxError::NonCritical(error) => {
                    tracing::error!("{} non-critical error: {error}", state.user);
                    net.writer
                        .write_error(convert_non_critical(error))
                        .await?;
                }

                TcpFluxError::Critical(error) => {
                    tracing::error!("{} critical error: {error}", state.user);
                    net.writer
                        .write_error(convert_critical(error))
                        .await?;
                }

                _ => return Err(e),
            };
        }
    }
}
