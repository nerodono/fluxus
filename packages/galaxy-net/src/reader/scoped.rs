use std::{
    future::{
        poll_fn,
        Future,
    },
    io,
    num::{
        NonZeroU16,
        NonZeroUsize,
    },
    pin::Pin,
};

use galaxy_net_raw::{
    packet_type::{
        PacketFlags,
        PacketType,
    },
    related::Protocol,
};
use galaxy_shrinker::interface::Decompressor;
use tokio::io::{
    AsyncReadExt,
    ReadBuf,
};

use super::{
    impl_::GalaxyReader,
    ReadResult,
};
use crate::{
    __raw_impl,
    error::ReadError,
    schemas::{
        ForwardPacketDescriptor,
        ServerDescriptor,
        StartedServerDescriptor,
    },
};

pub trait ReadExt: AsyncReadExt + Unpin {}
impl<T: AsyncReadExt + Unpin> ReadExt for T {}

pub struct RawReader<'a, R, D> {
    pub(crate) stream: &'a mut GalaxyReader<R, D>,
}

pub struct ServerReader<'a, R, D> {
    pub(crate) raw: RawReader<'a, R, D>,
}

pub struct ClientReader<'a, R, D> {
    pub(crate) raw: RawReader<'a, R, D>,
}

pub struct CommonReader<'a, R, D> {
    pub(crate) raw: RawReader<'a, R, D>,
}

//

impl<'a, R: ReadExt, D: Decompressor> CommonReader<'a, R, D> {
    /// Read [`ForwardPacketDescriptor`] from the stream.
    pub async fn read_forward(
        &mut self,
        flags: PacketFlags,
        max_size: NonZeroUsize,
    ) -> ReadResult<ForwardPacketDescriptor> {
        let client_id = self.raw.read_client_id(flags).await?;
        let length = self.raw.read_length(flags).await? as usize;
        if length > max_size.get() {
            return Err(ReadError::TooLongBufferSize {
                expected: max_size,
                got: length,
            });
        }

        let mut buffer = self.raw.read_buffer(length).await?;

        if flags.intersects(PacketFlags::COMPRESSED) {
            let decompressor = self.raw.decompressor_mut();
            if let Some(dec_size) = decompressor.try_get_size(&buffer)
            {
                if dec_size > max_size {
                    return Err(ReadError::TooLongBufferSize {
                        expected: max_size,
                        got: length,
                    });
                }

                // TODO: don't check size twice
                let buffer_allocator: fn(
                    Option<NonZeroUsize>,
                )
                    -> Option<_> = |determined_size| {
                    determined_size
                        .map(|c| Vec::with_capacity(c.get()))
                };

                buffer = decompressor
                    .try_decompress(&buffer, buffer_allocator)?;
            } else {
                return Err(ReadError::NoDecompressionSize);
            }
        }

        Ok(ForwardPacketDescriptor { client_id, buffer })
    }
}

//

impl<'a, R: ReadExt, D> ClientReader<'a, R, D> {
    /// Read [`StartedServerDescriptor`] from the stream.
    pub async fn read_server(
        &mut self,
        flags: PacketFlags,
    ) -> ReadResult<StartedServerDescriptor> {
        if flags.intersects(PacketFlags::SHORT_C) {
            Ok(StartedServerDescriptor { at_port: None })
        } else if flags.intersects(PacketFlags::SHORT_C) {
            Ok(StartedServerDescriptor {
                at_port: NonZeroU16::new(
                    self.raw.read_u8().await? as u16,
                ),
            })
        } else {
            Ok(StartedServerDescriptor {
                at_port: NonZeroU16::new(self.raw.read_u16().await?),
            })
        }
    }
}

//

impl<'a, R: ReadExt, D> ServerReader<'a, R, D> {
    /// Read [`ServerDescriptor`] from the stream.
    pub async fn read_server(
        &mut self,
        flags: PacketFlags,
    ) -> ReadResult<ServerDescriptor> {
        let protocol = if flags.intersects(PacketFlags::SHORT_C) {
            Protocol::Tcp
        } else {
            let supplied = self.raw.read_u8().await?;
            Protocol::try_from(supplied).map_err(|()| {
                ReadError::UnknownProtocol { supplied }
            })?
        };

        let port = if flags.intersects(PacketFlags::SHORT) {
            None
        } else if flags.intersects(PacketFlags::COMPRESSED) {
            NonZeroU16::new(self.raw.read_u8().await? as u16)
        } else {
            NonZeroU16::new(self.raw.read_u16().await?)
        };

        Ok(ServerDescriptor { protocol, port })
    }
}

//

impl<'a, R: ReadExt, D> RawReader<'a, R, D> {
    /// Same as [`RawReader::read_packet_type`], but returns
    /// [`ReadError::Unexpected`] if read packet was not
    /// `packet_type`.
    pub async fn expect_packet(
        &mut self,
        packet_type: PacketType,
    ) -> ReadResult<(PacketType, PacketFlags)> {
        self.read_packet_type()
            .await
            .and_then(|(got, flags)| {
                if got != packet_type {
                    Err(ReadError::Unexpected {
                        got,
                        expected: packet_type,
                    })
                } else {
                    Ok((got, flags))
                }
            })
    }

    /// Read and decode packet type.
    pub async fn read_packet_type(
        &mut self,
    ) -> ReadResult<(PacketType, PacketFlags)> {
        let packed = self.read_u8().await?;
        PacketType::try_decode(packed)
            .ok_or(ReadError::UnknownPacketType { type_: packed })
    }

    /// Same as [`RawReader::read_variadic`], but with
    /// pre-filled `need` argument with
    /// [`PacketFlags::SHORT`]
    pub async fn read_length(
        &mut self,
        flags: PacketFlags,
    ) -> io::Result<u16> {
        self.read_variadic(flags, PacketFlags::SHORT)
            .await
    }

    /// Same as [`RawReader::read_variadic`], but with
    /// pre-filled `need` argument with
    /// [`PacketFlags::SHORT_C`].
    pub async fn read_client_id(
        &mut self,
        flags: PacketFlags,
    ) -> io::Result<u16> {
        self.read_variadic(flags, PacketFlags::SHORT_C)
            .await
    }

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
