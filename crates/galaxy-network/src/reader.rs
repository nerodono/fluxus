use std::{
    future::{
        poll_fn,
        Future,
    },
    io,
    pin::Pin,
};

use galaxy_shrinker::interface::Decompressor;
use tokio::io::{
    AsyncReadExt,
    ReadBuf,
};

use crate::{
    error::ReadError,
    raw::{
        Packet,
        PacketFlags,
    },
};

pub type ReadResult<T> = Result<T, ReadError>;

pub trait Read: Unpin + AsyncReadExt {}
impl<T: Unpin + AsyncReadExt> Read for T {}

pub struct GalaxyReader<R, D> {
    raw: R,
    decompressor: D,
}

impl<R: Read, D> GalaxyReader<R, D> {
    pub async fn read_packet_type(&mut self) -> ReadResult<Packet> {
        Packet::from_u8(self.read_u8().await?)
            .ok_or(ReadError::UnknownPacket)
    }

    /// Reads `no` bytes into the start of `into` vector.
    /// On success `into` vector will be size of `no`.
    pub async fn read_buffer_into(
        &mut self,
        into: &mut Vec<u8>,
        no: usize,
    ) -> io::Result<()> {
        let mut buffer =
            ReadBuf::uninit(&mut into.spare_capacity_mut()[..no]);
        while buffer.filled().len() < no {
            poll_fn(|cx| {
                Pin::new(&mut self.raw).poll_read(cx, &mut buffer)
            })
            .await?;
        }

        // SAFETY: this is safe since previous loop would exit only
        // if `no` bytes is filled (properly initialized).
        unsafe { into.set_len(no) };

        Ok(())
    }

    /// Read variadic value from the stream. Reads single
    /// byte if `flags` contains `needed`, otherwise reads
    /// two bytes.
    #[inline]
    pub async fn read_variadic(
        &mut self,
        flags: PacketFlags,
        needed: PacketFlags,
    ) -> io::Result<u16> {
        if flags.contains(needed) {
            self.raw.read_u8().await.map(|u| u as _)
        } else {
            self.raw.read_u16().await
        }
    }

    /// Same as [`GalaxyReader::read_bytes_prefixed`] but
    /// returns lossy decoded utf8 String.
    pub async fn read_string_prefixed(&mut self) -> io::Result<String> {
        let data = self.read_bytes_prefixed().await?;
        Ok(String::from_utf8_lossy(&data).into_owned())
    }

    /// Read arbitrary amount of bytes from the underlying
    /// stream. Length determinition is based on single byte
    /// prefix (1 byte length + buffer of that length).
    #[inline]
    pub async fn read_bytes_prefixed(&mut self) -> io::Result<Vec<u8>> {
        let length = self.read_u8().await? as usize;
        let mut buf = Vec::with_capacity(length);
        self.read_buffer_into(&mut buf, length).await?;
        Ok(buf)
    }

    /// Read U16 from the underlying stream.
    #[inline]
    pub fn read_u16(
        &mut self,
    ) -> impl Future<Output = io::Result<u16>> + '_ {
        self.raw.read_u16_le()
    }

    /// Read U8 from the underlying stream.
    #[inline]
    pub fn read_u8(
        &mut self,
    ) -> impl Future<Output = io::Result<u8>> + '_ {
        self.raw.read_u8()
    }
}

impl<R, D> GalaxyReader<R, D> {
    pub const fn new(raw: R, decompressor: D) -> Self {
        Self { raw, decompressor }
    }
}