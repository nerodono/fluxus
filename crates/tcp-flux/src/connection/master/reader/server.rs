use std::{
    io,
    num::NonZeroU16,
};

use crate::{
    connection::{
        master::payloads::create_tcp_request::CreateTcpRequest,
        traits::RawRead,
    },
    types::pkt_base::PktFlags,
};

pub struct MasterServerReader<'a, R> {
    pub(crate) reader: &'a mut R,
}

impl<'a, R: RawRead> MasterServerReader<'a, R> {
    /// Reads `create tcp proxy` request payload.
    /// If remote user passes `0` as specific port, then
    /// `specific_port` would be left as [`None`]
    ///
    /// ### How flags affect the behavior
    /// - [`PktFlags::FLAG0`]: if set, port will not be
    ///   read, leaving [`None`] value instead
    pub async fn read_create_tcp_request(
        &mut self,
        flags: PktFlags,
    ) -> io::Result<CreateTcpRequest> {
        if flags.contains(PktFlags::FLAG0) {
            Ok(CreateTcpRequest {
                specific_port: None,
            })
        } else {
            Ok(CreateTcpRequest {
                specific_port: NonZeroU16::new(self.reader.read_u16_le().await?),
            })
        }
    }
}
