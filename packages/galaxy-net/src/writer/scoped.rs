use std::{
    future::Future,
    io::{
        self,
        IoSlice,
    },
    num::NonZeroUsize,
    ptr,
};

use galaxy_net_raw::{
    error::Failure,
    packet_type::{
        PacketFlags,
        PacketType,
    },
    related::Protocol,
};
use galaxy_shrinker::interface::Compressor;
use tokio::io::AsyncWriteExt;

use super::GalaxyWriter;
use crate::{
    __raw_impl,
    schemas::{
        ForwardPacketDescriptor,
        PingResponseDescriptor,
        ServerDescriptor,
        StartedServerDescriptor,
    },
    utils::encode_forward_header,
};

pub trait WriteExt: AsyncWriteExt + Unpin {}
impl<T: AsyncWriteExt + Unpin> WriteExt for T {}

pub struct RawWriter<'a, W, C> {
    pub(crate) stream: &'a mut GalaxyWriter<W, C>,
}

pub struct ClientWriter<'a, W, C> {
    pub(crate) raw: RawWriter<'a, W, C>,
}

pub struct ServerWriter<'a, W, C> {
    pub(crate) raw: RawWriter<'a, W, C>,
}

pub struct CommonWriter<'a, W, C> {
    pub(crate) raw: RawWriter<'a, W, C>,
}

//

impl<'a, W: WriteExt, C> ServerWriter<'a, W, C> {
    /// Write [`PingResponseDescriptor`] to the stream
    /// (including packet type).
    pub async fn write_ping(
        &mut self,
        descriptor: PingResponseDescriptor<'_>,
    ) -> io::Result<()> {
        let buffer = descriptor
            .read_buffer
            .try_into()
            .unwrap_or(u16::MAX);
        self.raw
            .write_two_bufs(
                &[
                    PacketType::Ping.encode_ident(),
                    descriptor.compression_level,
                    descriptor.compression_method as u8,
                    (buffer & 0xff) as u8,
                    (buffer >> 8) as u8,
                    descriptor.server_name.len() as u8,
                ],
                descriptor.server_name.as_bytes(),
            )
            .await
            .map(|_| ())
    }

    /// Write `Connect` packet to the stream.
    pub async fn write_connect(
        &mut self,
        client_id: u16,
    ) -> io::Result<()> {
        self.raw
            .write_client_id(PacketType::Connect, client_id)
            .await
    }

    /// Write failure packet to the stream.
    pub async fn write_failure(
        &mut self,
        failure: Failure,
    ) -> io::Result<()> {
        self.raw
            .write_buffer(&[
                PacketType::Failure.encode_ident(),
                failure as u8,
            ])
            .await
    }

    /// Write [`StartedServerDescriptor`] to the stream
    /// (including packet type)
    pub async fn write_server(
        &mut self,
        descriptor: StartedServerDescriptor,
    ) -> io::Result<()> {
        let at_port = descriptor.at_port.map(|p| p.get());
        match at_port {
            Some(port @ ..=0xff) => {
                self.raw
                    .write_buffer(&[
                        PacketType::Server.encode(PacketFlags::SHORT),
                        port as u8,
                    ])
                    .await
            }
            Some(port) => {
                self.raw
                    .write_buffer(&[
                        PacketType::Server.encode_ident(),
                        (port & 0xff) as u8,
                        (port >> 8) as u8,
                    ])
                    .await
            }
            None => {
                self.raw
                    .write_buffer(&[PacketType::Server
                        .encode(PacketFlags::SHORT_C)])
                    .await
            }
        }
    }
}

//

impl<'a, W: WriteExt, C: Compressor> CommonWriter<'a, W, C> {
    #[inline(always)]
    async fn write_forward_impl(
        &mut self,
        descriptor: ForwardPacketDescriptor,
        flags: PacketFlags,
    ) -> io::Result<()> {
        let (hdr, hdr_len) = encode_forward_header(
            descriptor.buffer.len() as u16,
            descriptor.client_id,
            flags,
        );
        self.raw
            .write_two_bufs(
                &hdr[..hdr_len as usize],
                &descriptor.buffer,
            )
            .await
            .map(|_| ())
    }

    /// Write [`ForwardPacketDescriptor`] into the stream
    /// (including packet type).
    ///
    /// Note: according to the `compression_threshold`
    /// packet will be compressed, if:
    /// - `compress_threshold` is not [`None`]
    /// - `descriptor.buffer.len() >= compression_threshold`
    /// - compressed size is lesser than
    ///   `descriptor.buffer.len()`.
    ///
    /// TODO: Implement compression result reporting
    pub async fn write_forward(
        &mut self,
        descriptor: ForwardPacketDescriptor,
        compress_threshold: Option<NonZeroUsize>,
    ) -> io::Result<()> {
        // Is it possible to refactor according to all conditions?
        match compress_threshold {
            Some(threshold) => {
                if descriptor.buffer.len() >= threshold.get() {
                    let mut dst =
                        Vec::with_capacity(descriptor.buffer.len());

                    match self
                        .raw
                        .compressor_mut()
                        .try_compress(&descriptor.buffer, &mut dst)
                    {
                        Ok(_compressed_size) => {
                            // TODO: Check `compressed_size <
                            // descriptor.buffer.len()`
                            // (Yeah, this is possible due to nature
                            // of [`Vec::with_capacity`])
                            self.write_forward_impl(
                                ForwardPacketDescriptor {
                                    buffer: dst,
                                    ..descriptor
                                },
                                PacketFlags::COMPRESSED,
                            )
                            .await
                        }

                        // TODO: handle `_e`
                        Err(_e) => {
                            self.write_forward_impl(
                                descriptor,
                                PacketFlags::empty(),
                            )
                            .await
                        }
                    }
                } else {
                    self.write_forward_impl(
                        descriptor,
                        PacketFlags::empty(),
                    )
                    .await
                }
            }
            None => {
                self.write_forward_impl(
                    descriptor,
                    PacketFlags::empty(),
                )
                .await
            }
        }
    }
}

impl<'a, W: WriteExt, C> CommonWriter<'a, W, C> {
    /// Same as [`RawWriter::write_client_id`], but with
    /// `packet_type` = [`PacketType::Disconnect`]
    pub async fn write_disconnect(
        &mut self,
        client_id: u16,
    ) -> io::Result<()> {
        self.raw
            .write_client_id(PacketType::Disconnect, client_id)
            .await
    }
}

//

impl<'a, W: WriteExt, C> ClientWriter<'a, W, C> {
    /// Write `Ping` request to the server
    pub async fn write_ping(&mut self) -> io::Result<()> {
        self.raw
            .write_buffer(&[PacketType::Ping.encode_ident()])
            .await
    }

    /// Write server descriptor to the stream (including
    /// packet type).
    pub async fn write_server(
        &mut self,
        descriptor: ServerDescriptor,
    ) -> io::Result<()> {
        let mut buf = [0; 4];
        let mut flags = PacketFlags::empty();
        let mut offset = 1;

        offset += if descriptor.protocol == Protocol::Tcp {
            flags |= PacketFlags::SHORT_C;
            0
        } else {
            buf[offset] = descriptor.protocol as u8;
            1
        };

        let port = descriptor.port.map(|i| i.get());
        offset += match port {
            Some(0) | None => {
                flags |= PacketFlags::SHORT;
                0
            }

            Some(p @ ..=0xff) => {
                buf[offset] = p as u8;
                flags |= PacketFlags::COMPRESSED;
                1
            }
            Some(p) => {
                buf[offset] = (p & 0xff) as u8;
                buf[offset + 1] = (p >> 8) as u8;
                2
            }
        };

        self.raw.write_buffer(&buf[..offset]).await
    }
}

//

impl<'a, W: WriteExt, C> RawWriter<'a, W, C> {
    /// Write arbitrary packet that contains only
    /// `client_id` field.
    pub async fn write_client_id(
        &mut self,
        packet_type: PacketType,
        id: u16,
    ) -> io::Result<()> {
        let flags: PacketFlags;
        if id <= 0xff {
            flags = PacketFlags::SHORT_C;
            self.write_buffer(&[packet_type.encode(flags), id as u8])
                .await
        } else {
            flags = PacketFlags::empty();
            self.write_buffer(&[
                packet_type.encode(flags),
                (id & 0xff) as u8,
                (id >> 8) as u8,
            ])
            .await
        }
    }

    /// Writes two buffers in at least 1 syscall (fast
    /// path, likely to work).
    ///
    /// Note: if target writer does not support vectored
    /// writes, then intermediate buffer allocated
    /// and data will be copied inside it.
    pub async fn write_two_bufs(
        &mut self,
        prepend: &[u8],
        append: &[u8],
    ) -> io::Result<bool> {
        // :D
        fn true_((): ()) -> bool {
            true
        }

        let plen = prepend.len();
        let alen = append.len();
        let total = plen + alen;

        if !self.stream().is_write_vectored() {
            let mut buf: Vec<u8> = Vec::with_capacity(total);

            // Q: Possibly it could be done without any unsafe and
            // buffer pre-filling?
            unsafe {
                let spare = buf.spare_capacity_mut();
                let spare_ptr = spare.as_mut_ptr();
                ptr::copy_nonoverlapping(
                    prepend.as_ptr(),
                    spare_ptr as *mut _,
                    plen,
                );
                ptr::copy_nonoverlapping(
                    append.as_ptr(),
                    spare_ptr.add(plen) as *mut _,
                    alen,
                );

                buf.set_len(total);
            }

            return self
                .stream_mut()
                .write_all(&buf)
                .await
                .map(true_);
        }

        let slices = [IoSlice::new(prepend), IoSlice::new(append)];
        let mut written = 0;

        while written < total {
            let wrote =
                self.stream_mut().write_vectored(&slices).await?;
            written += wrote;
            if written >= plen {
                return self
                    .stream_mut()
                    .write_all(&append[(written - plen)..])
                    .await
                    .map(true_);
            }
        }

        Ok(true)
    }

    /// Write supplied buffer into the stream.
    pub fn write_buffer<'k>(
        &'k mut self,
        buffer: &'k [u8],
    ) -> impl Future<Output = io::Result<()>> + 'k {
        self.stream_mut().write_all(buffer)
    }
}

impl<'a, W, C> RawWriter<'a, W, C> {
    __raw_impl! { @stream<W> stream.stream }

    __raw_impl! { @compressor<C> stream.compressor }
}
