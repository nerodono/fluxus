use crate::{
    connection::traits::RawRead,
    error::PktBaseReadError,
    types::pkt_base::PktBase,
};

pub struct MasterReader<R> {
    inner: R,
}

#[must_use = "Read payload of the packet or use .no_payload()"]
pub struct MasterPayloadReader<'a, R> {
    inner: &'a mut R,
}

impl<'a, R: RawRead> MasterPayloadReader<'a, R> {
    /// Simply discard payload, since packet doesn't need it
    pub const fn no_payload(self) {}
}

impl<R: RawRead> MasterReader<R> {
    /// Read base header of the packet and start reading
    /// it's payload
    pub async fn next_packet(
        &mut self,
    ) -> Result<(MasterPayloadReader<'_, R>, PktBase), PktBaseReadError> {
        let int = self.inner.read_u8().await?;
        let base =
            PktBase::try_decode(int).ok_or(PktBaseReadError::InvalidType(int))?;
        Ok((
            MasterPayloadReader {
                inner: &mut self.inner,
            },
            base,
        ))
    }

    pub const fn new(inner: R) -> Self {
        Self { inner }
    }
}
