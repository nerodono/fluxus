use std::borrow::Cow;

use tokio::io::ReadBuf;

use crate::{
    connection::{
        master::payloads::info::InfoPayload,
        traits::RawRead,
    },
    error::ReadError,
};

type ReadResult<T> = Result<T, ReadError>;

pub struct MasterClientReader<'a, R> {
    pub(crate) reader: &'a mut R,
}

impl<'a, R: RawRead> MasterClientReader<'a, R> {
    pub async fn read_info(&mut self) -> ReadResult<InfoPayload<'static>> {
        Ok(InfoPayload {
            server_name: Cow::Owned(self.read_string().await?),
        })
    }
}

impl<'a, R: RawRead> MasterClientReader<'a, R> {
    async fn read_string(&mut self) -> ReadResult<String> {
        let str_len = self.reader.read_u8().await?;
        let buf = self.read_buffer(str_len as usize).await?;

        match String::from_utf8(buf) {
            Ok(s) => Ok(s),
            Err(_) => Err(ReadError::InvalidString),
        }
    }

    async fn read_buffer(&mut self, size: usize) -> ReadResult<Vec<u8>> {
        let mut vec = Vec::with_capacity(size);
        {
            let mut buf = ReadBuf::uninit(&mut vec.spare_capacity_mut()[..size]);

            while buf.filled().len() != size {
                if self.reader.read_buf(&mut buf).await? == 0 {
                    return Err(ReadError::EndOfStream);
                }
            }
        }

        unsafe { vec.set_len(size) };

        Ok(vec)
    }
}
