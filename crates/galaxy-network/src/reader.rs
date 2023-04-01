use std::{
    future::{
        poll_fn,
        Future,
    },
    io,
    num::NonZeroUsize,
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
        Protocol,
        Rights,
    },
};

pub type ReadResult<T> = Result<T, ReadError>;

pub trait Read: Unpin + AsyncReadExt {}
impl<T: Unpin + AsyncReadExt> Read for T {}

pub struct GalaxyReader<R, D> {
    raw: R,
    decompressor: D,
}

impl<R: Read, D: Decompressor> GalaxyReader<R, D> {
    pub async fn try_read_compressed(
        &mut self,
        length: usize,
        validate_size: impl FnOnce(NonZeroUsize) -> bool,
    ) -> ReadResult<Vec<u8>> {
        let mut compressed = Vec::with_capacity(length);
        self.read_buffer_into(&mut compressed, length)
            .await?;

        let Some(size) = self.decompressor.try_get_decompressed_size(&compressed) else {
            return Err(ReadError::FailedToRetrieveUncompressedSize);
        };
        if !validate_size(size) {
            return Err(ReadError::FailedToDecompress);
        }

        self.decompressor
            .try_decompress(&compressed, size.get())
            .ok_or(ReadError::FailedToDecompress)
    }
}

impl<R: Read, D> GalaxyReader<R, D> {
    pub async fn read_rights(&mut self) -> ReadResult<Rights> {
        let bits = self.read_u8().await?;
        Rights::from_bits(bits).ok_or(ReadError::InvalidRights { bits })
    }

    pub async fn skip_n_bytes<const CHUNK_SIZE: usize>(
        &mut self,
        nbytes: usize,
    ) -> io::Result<()> {
        let mut skipped = 0;
        let mut chunk = [0; CHUNK_SIZE];

        while skipped < nbytes {
            let current_chunk = (nbytes - skipped).min(CHUNK_SIZE);
            self.raw
                .read_exact(&mut chunk[..current_chunk])
                .await?;
            skipped += current_chunk;
        }

        Ok(())
    }

    pub async fn read_protocol_type(
        &mut self,
        flags: PacketFlags,
    ) -> ReadResult<Protocol> {
        if flags.intersects(PacketFlags::COMPRESSED) {
            Ok(Protocol::Tcp)
        } else if flags.intersects(PacketFlags::SHORT) {
            Ok(Protocol::Udp)
        } else if flags.intersects(PacketFlags::SHORT_CLIENT) {
            Ok(Protocol::Http)
        } else {
            // FIXME: Custom protocol retrieval
            Ok(Protocol::Tcp)
        }
    }

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
