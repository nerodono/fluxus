use tcp_flux::{
    connection::{
        master::{
            reader::server::MasterServerReader,
            writer::server::MasterServerWriter,
        },
        traits::{
            RawRead,
            RawWrite,
        },
    },
    types::pkt_base::{
        PktBase,
        PktType,
    },
};

use super::{
    atom::Atom,
    connection::ConnectionState,
};
use crate::{
    error::CriticalError,
    protocols::tcp_flux::error::{
        TcpFluxError,
        TcpFluxResult,
    },
};

pub async fn route_packet<R, W>(
    reader: MasterServerReader<'_, R>,
    writer: &mut MasterServerWriter<W>,
    state: &mut ConnectionState<'_>,

    pkt: PktBase,
) -> TcpFluxResult<()>
where
    R: RawRead,
    W: RawWrite,
{
    use PktType as P;
    let atom = Atom {
        reader,
        writer,
        state,
        flags: pkt.flags,
    };

    match pkt.type_ {
        P::Authenticate => atom.authenticate().await,
        P::ReqInfo => atom.req_info().await,
        P::Disconnect => atom.disconnect().await,
        P::CreateHttp => atom.create_http().await,
        P::CreateTcp => atom.create_tcp().await,

        P::Connected | P::Error | P::UpdateRights => {
            Err(TcpFluxError::Critical(CriticalError::UnexpectedPacket))
        }
    }
}
