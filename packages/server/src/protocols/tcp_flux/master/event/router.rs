use tcp_flux::connection::{
    master::writer::server::MasterServerWriter,
    traits::RawWrite,
};

use crate::protocols::tcp_flux::{
    error::TcpFluxResult,
    events::master::MasterEvent,
    master::network::connection::ConnectionState,
};

pub async fn route_event<W>(
    event: MasterEvent,
    writer: &mut MasterServerWriter<W>,
    state: &mut ConnectionState<'_>,
) -> TcpFluxResult<()>
where
    W: RawWrite,
{
    todo!()
}
