use tcp_flux::connection::{
    master::{
        reader::server::MasterServerReader,
        writer::server::MasterServerWriter,
    },
    traits::{
        RawRead,
        RawWrite,
    },
};

use super::connection::ConnectionState;
use crate::protocols::tcp_flux::error::TcpFluxResult;

/// Indivisible scope of connection: actual packet handling
///
/// ```text
///  Connection*
/// ------------>
/// |----|------|
///  ^^^^ ^^^^^^
///  Atom  Atom
/// ```
/// * Connection here is an actual connection, not the
///   **state of connection**
pub struct Atom<'r, 'cfg, R, W> {
    state: &'r mut ConnectionState<'cfg>,

    writer: &'r mut MasterServerWriter<W>,
    reader: MasterServerReader<'r, R>,
}

impl<'r, 'cfg, R: RawRead, W: RawWrite> Atom<'r, 'cfg, R, W> {
    pub async fn authenticate(&mut self) -> TcpFluxResult<()> {
        todo!()
    }

    pub async fn req_info(&mut self) -> TcpFluxResult<()> {
        todo!()
    }

    pub async fn disconnect(&mut self) -> TcpFluxResult<()> {
        todo!()
    }
}

impl<'r, 'cfg, R, W> Atom<'r, 'cfg, R, W> {
    pub fn new(
        state: &'r mut ConnectionState<'cfg>,
        reader: MasterServerReader<'r, R>,
        writer: &'r mut MasterServerWriter<W>,
    ) -> Self {
        Self {
            state,
            reader,
            writer,
        }
    }
}
