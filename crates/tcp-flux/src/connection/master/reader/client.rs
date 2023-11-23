use std::io;

use crate::connection::{
    master::payloads::info::InfoPayload,
    traits::RawRead,
};

pub struct MasterClientReader<'a, R> {
    pub(crate) reader: &'a mut R,
}

impl<'a, R: RawRead> MasterClientReader<'a, R> {
    pub async fn read_info(&mut self) -> io::Result<InfoPayload<'static>> {
        todo!()
    }
}

impl<'a, R: RawRead> MasterClientReader<'a, R> {
    async fn read_string(&mut self) -> io::Result<String> {
        let str_len = self.reader.read_u8().await?;
        todo!()
    }
}
