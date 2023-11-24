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

use super::connection::Connection;
use crate::protocols::tcp_flux::error::TcpFluxResult;

pub struct Atom<'a, R, W> {
    connection: &'a mut Connection,

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
        connection: &'a mut Connection,
        reader: MasterServerReader<'a, R>,
        writer: &'a mut MasterServerWriter<W>,
    ) -> Self {
        Self {
            connection,
            reader,
            writer,
        }
    }
}
