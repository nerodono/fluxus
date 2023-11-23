use std::marker::PhantomData;

use super::{
    client::MasterClientReader,
    server::MasterServerReader,
};
use crate::{
    connection::traits::RawRead,
    error::PktBaseReadError,
    types::pkt_base::PktBase,
};

pub enum Client {}
pub enum Server {}

mod details {
    pub trait Sealed {}
}

pub trait ReaderSide: details::Sealed {
    type Target<'a, R: 'a>;

    fn create<R: RawRead>(reader: &mut R) -> Self::Target<'_, R>;
}

impl details::Sealed for Client {}
impl details::Sealed for Server {}

impl ReaderSide for Client {
    type Target<'a, R: 'a> = MasterClientReader<'a, R>;

    fn create<R: RawRead>(reader: &mut R) -> Self::Target<'_, R> {
        MasterClientReader { reader }
    }
}

impl ReaderSide for Server {
    type Target<'a, R: 'a> = MasterServerReader<'a, R>;

    fn create<R: RawRead>(reader: &mut R) -> Self::Target<'_, R> {
        MasterServerReader { reader }
    }
}

pub struct MasterReader<R, S: ReaderSide> {
    reader: R,
    _phantom: PhantomData<S>,
}

impl<R: RawRead, S: ReaderSide> MasterReader<R, S> {
    pub async fn next_packet(
        &mut self,
    ) -> Result<(PktBase, S::Target<'_, R>), PktBaseReadError> {
        let int = self.reader.read_u8().await?;
        let base =
            PktBase::try_decode(int).ok_or(PktBaseReadError::InvalidType(int))?;
        Ok((base, S::create(&mut self.reader)))
    }
}

impl<R, S: ReaderSide> MasterReader<R, S> {
    pub const fn new(reader: R) -> Self {
        Self {
            reader,
            _phantom: PhantomData,
        }
    }
}
