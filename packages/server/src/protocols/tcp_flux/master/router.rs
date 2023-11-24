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
            TcpFluxError,
            TcpFluxResult,
        },
        master::atom::Atom,
    },
};

pub struct Router<'a, R, W> {
    sides: Sides<R, W>,
    connection_state: ConnectionState<'a>,
}

impl<'a, R: RawRead, W: RawWrite> Router<'a, R, W> {
    pub async fn serve(mut self) -> TcpFluxResult<()> {
        use PktType as P;
        loop {
            let net = &mut self.sides;

            let (pkt, reader) = net.reader.next_packet().await?;
            let mut atom =
                Atom::new(&mut self.connection_state, reader, &mut net.writer);

            let result = match pkt.type_ {
                P::Authenticate => atom.authenticate().await,
                P::ReqInfo => atom.req_info().await,
                P::Disconnect => atom.disconnect().await,
                P::Connected | P::Error => {
                    Err(TcpFluxError::Critical(CriticalError::UnexpectedPacket))
                }
            };

            if let Err(e) = result {
                match e {
                    TcpFluxError::NonCritical(error) => {
                        tracing::error!(
                            "{} non-critical error: {error}",
                            self.connection_state.user
                        );
                        todo!();
                    }

                    TcpFluxError::Critical(crit) => {
                        tracing::error!(
                            "{} critical error: {crit}",
                            self.connection_state.user
                        );
                        todo!();
                    }

                    _ => return Err(e),
                }
            }
        }
    }
}

impl<'a, R, W> Router<'a, R, W> {
    pub const fn new(
        connection_state: ConnectionState<'a>,
        sides: Sides<R, W>,
    ) -> Self {
        Self {
            connection_state,
            sides,
        }
    }
}
