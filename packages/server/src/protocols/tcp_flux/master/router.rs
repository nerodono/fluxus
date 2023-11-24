use tcp_flux::{
    connection::traits::{
        RawRead,
        RawWrite,
    },
    types::pkt_base::PktType,
};

use super::connection::{
    Connection,
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

pub struct Router<R, W> {
    sides: Sides<R, W>,
    connection: Connection,
}

impl<R: RawRead, W: RawWrite> Router<R, W> {
    pub async fn serve(mut self) -> TcpFluxResult<()> {
        use PktType as P;
        loop {
            let conn = &mut self.connection;
            let net = &mut self.sides;

            let (pkt, reader) = net.reader.next_packet().await?;
            let mut atom = Atom::new(conn, reader, &mut net.writer);

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
                        todo!();
                    }

                    TcpFluxError::Critical(crit) => {
                        todo!();
                    }

                    _ => return Err(e),
                }
            }
        }
    }
}

impl<R, W> Router<R, W> {
    pub const fn new(connection: Connection, sides: Sides<R, W>) -> Self {
        Self { connection, sides }
    }
}
