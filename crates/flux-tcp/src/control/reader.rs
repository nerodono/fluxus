use std::{
    future::Future,
    io,
};

use tokio::io::{
    AsyncReadExt,
    BufReader,
    ReadBuf,
};

use crate::{
    error::{
        ControlReadError,
        ControlReadResult,
    },
    raw::Rights,
    traits::ComposeRead,
};

pub struct ControlReader<R> {
    inner: BufReader<R>,
}

impl<R: ComposeRead> ControlReader<R> {
    #[inline]
    pub fn read_u16(&mut self) -> impl Future<Output = io::Result<u16>> + '_ {
        self.inner.read_u16_le()
    }

    #[inline]
    pub fn read_u8(&mut self) -> impl Future<Output = io::Result<u8>> + '_ {
        self.inner.read_u8()
    }

    pub async fn read_rights(&mut self) -> ControlReadResult<Rights> {
        let bits = self.read_u8().await?;
        Rights::from_bits(bits).ok_or(ControlReadError::InvalidRights(bits))
    }

    pub async fn read_bytes_prefixed(
        &mut self,
    ) -> ControlReadResult<Vec<u8>> {
        let len = self.read_u8().await? as usize;
        self.read_exact(len).await
    }

    pub async fn read_exact(
        &mut self,
        nbytes: usize,
    ) -> ControlReadResult<Vec<u8>> {
        let mut vec = Vec::with_capacity(nbytes);

        {
            let mut buffer =
                ReadBuf::uninit(&mut vec.spare_capacity_mut()[..nbytes]);

            while buffer.remaining() != 0 {
                let read = self.inner.read_buf(&mut buffer).await?;
                if read == 0 {
                    return Err(ControlReadError::Disconnected);
                }
            }
        }

        unsafe { vec.set_len(nbytes) };

        Ok(vec)
    }
}

impl<R: ComposeRead> ControlReader<R> {
    pub fn with_capacity(inner: R, capacity: usize) -> Self {
        Self {
            inner: BufReader::with_capacity(capacity, inner),
        }
    }
}
