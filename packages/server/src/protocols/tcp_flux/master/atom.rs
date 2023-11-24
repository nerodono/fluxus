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
pub struct Atom<'a, R, W> {
    state: &'a mut ConnectionState,

    writer: &'a mut MasterServerWriter<W>,
    reader: MasterServerReader<'a, R>,
}

impl<'a, R: RawRead, W: RawWrite> Atom<'a, R, W> {
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

impl<'a, R, W> Atom<'a, R, W> {
    pub fn new(
        state: &'a mut ConnectionState,
        reader: MasterServerReader<'a, R>,
        writer: &'a mut MasterServerWriter<W>,
    ) -> Self {
        Self {
            state,
            reader,
            writer,
        }
    }
}
