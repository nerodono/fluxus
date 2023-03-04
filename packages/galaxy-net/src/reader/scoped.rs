use std::{
    future::{
        poll_fn,
        Future,
    },
    io,
    pin::Pin,
};

use galaxy_net_raw::packet_type::PacketFlags;
use tokio::io::{
    AsyncReadExt,
    ReadBuf,
};

use super::impl_::GalaxyReader;
use crate::__raw_impl;

pub trait ReadExt: AsyncReadExt + Unpin {}
impl<T: AsyncReadExt + Unpin> ReadExt for T {}

pub struct RawReader<'a, R, D> {
    pub(crate) stream: &'a mut GalaxyReader<R, D>,
}

impl<'a, R: ReadExt, D> RawReader<'a, R, D> {
    /// Same as [`ReaderRaw::read_buffer_into`] but without
    /// the `into` buffer, just some length.
    ///
    /// Note: Internally it calls the
    /// [`ReaderRaw::read_buffer_into`], so all requirements
    /// and notes will be same for this method.
    pub async fn read_buffer(
        &mut self,
        length: usize,
    ) -> io::Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(length);
        self.read_buffer_into(&mut buf, length)
            .await
            .map(|()| buf)
    }

    /// Reads buffer into the `into`'s vector without
    /// pre-filling. This is done through writing to the
    /// vector capacity buffer and then setting vector len
    /// to the initialized bytes count.
    ///
    /// Note: `no` is required because `Vec::with_capacity`
    /// can allocate more than requested.
    pub async fn read_buffer_into(
        &mut self,
        into: &mut Vec<u8>,
        no: usize,
    ) -> io::Result<()> {
        into.clear();

        {
            let mut read_buf =
                ReadBuf::uninit(&mut into.spare_capacity_mut()[..no]);
            while read_buf.filled().len() < no {
                poll_fn(|cx| {
                    Pin::new(self.stream_mut())
                        .poll_read(cx, &mut read_buf)
                })
                .await?;
            }
        }

        unsafe { into.set_len(no) };
        Ok(())
    }

    /// Read variadic value from the stream.
    ///
    /// - U8 will be read if `flags` contains `needed`
    /// - U16 will be read otherwise
    pub async fn read_variadic(
        &mut self,
        flags: PacketFlags,
        needed: PacketFlags,
    ) -> io::Result<u16> {
        if flags.contains(needed) {
            self.read_u8().await.map(|u| u as u16)
        } else {
            self.read_u16().await
        }
    }

    /// Read U16 from the stream
    pub fn read_u16(
        &mut self,
    ) -> impl Future<Output = io::Result<u16>> + '_ {
        self.stream_mut().read_u16()
    }

    /// Read U8 from the stream.
    pub fn read_u8(
        &mut self,
    ) -> impl Future<Output = io::Result<u8>> + '_ {
        self.stream_mut().read_u8()
    }
}

impl<R, D> RawReader<'_, R, D> {
    __raw_impl! { @stream<R> stream.stream }

    __raw_impl! { @decompressor<D> stream.decompressor }
}
