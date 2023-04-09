use std::{
    future::Future,
    io::{
        self,
        IoSlice,
    },
    num::NonZeroU16,
};

use galaxy_shrinker::interface::Compressor;
use tokio::io::AsyncWriteExt;

use crate::{
    descriptors::{
        CreateServerResponseDescriptor,
        PingResponseDescriptor,
    },
    raw::{
        ErrorCode,
        Packet,
        PacketFlags,
        PacketType,
        Protocol,
        Rights,
    },
    utils::encode_forward_header,
};

pub trait Write: Unpin + AsyncWriteExt {}
impl<T: Unpin + AsyncWriteExt> Write for T {}

/// Write side of the `galaxy` protocol.
/// This type is designed to contain mostly single-writes in
/// sake of performance.
pub struct GalaxyWriter<W, C> {
    raw: W,
    compressor: C,
}

pub struct GalaxyServerWriter<'a, W, C>(&'a mut GalaxyWriter<W, C>);
pub struct GalaxyClientWriter<'a, W, C>(&'a mut GalaxyWriter<W, C>);

impl<'a, W: Write, C> GalaxyClientWriter<'a, W, C> {
    pub async fn write_server_request(
        &mut self,
        protocol: Protocol,
        port: Option<NonZeroU16>,
    ) -> io::Result<()> {
        let flags = match protocol {
            Protocol::Http => PacketFlags::SHORT_CLIENT,
            Protocol::Tcp => PacketFlags::COMPRESSED,
            Protocol::Udp => PacketFlags::SHORT,
        };
        let port = port.map(NonZeroU16::get).unwrap_or(0);
        self.raw_mut()
            .write_all(&[
                Packet::new(PacketType::CreateServer, flags).encode(),
                (port & 0xff) as u8,
                (port >> 8) as u8,
            ])
            .await
    }

    pub async fn write_password_auth(
        &mut self,
        password: &str,
    ) -> io::Result<()> {
        self.0
            .write_two_bufs(
                &[
                    Packet::id(PacketType::AuthorizePassword).encode(),
                    password
                        .len()
                        .try_into()
                        .expect("Too long password"),
                ],
                password.as_bytes(),
            )
            .await
    }

    pub fn write_ping(
        &mut self,
    ) -> impl Future<Output = io::Result<()>> + '_ {
        self.raw_mut()
            .write_u8(Packet::id(PacketType::Ping).encode())
    }

    pub fn raw_mut(&mut self) -> &mut W {
        &mut self.0.raw
    }
}

impl<W: Write, C> GalaxyServerWriter<'_, W, C> {
    pub async fn write_update_rights(
        &mut self,
        new_rights: Rights,
    ) -> io::Result<()> {
        self.0
            .write_all(&[
                Packet::id(PacketType::UpdateRight).encode(),
                new_rights.bits(),
            ])
            .await
    }

    #[inline]
    pub fn write_connected(
        &mut self,
        id: u16,
    ) -> impl Future<Output = io::Result<()>> + '_ {
        self.0.write_client_id(id, PacketType::Connect)
    }

    pub async fn write_server(
        &mut self,
        descriptor: &CreateServerResponseDescriptor,
    ) -> io::Result<()> {
        if let Some(port) = descriptor.port.map(NonZeroU16::get) {
            self.0
                .write_all(&[
                    Packet::id(PacketType::CreateServer).encode(),
                    (port & 0xff) as u8,
                    (port >> 8) as u8,
                ])
                .await
        } else {
            self.0
                .write_all(&[Packet::new(
                    PacketType::CreateServer,
                    PacketFlags::COMPRESSED,
                )
                .encode()])
                .await
        }
    }

    pub async fn write_ping(
        &mut self,
        descriptor: &PingResponseDescriptor<'_>,
    ) -> io::Result<()> {
        let buf_read = descriptor.buffer_read.get() as u16;
        self.0
            .write_two_bufs(
                &[
                    Packet::id(PacketType::Ping).encode(),
                    descriptor.compression.algorithm as u8,
                    descriptor.compression.level.get(),
                    (buf_read & 0xff) as u8,
                    (buf_read >> 8) as u8,
                    descriptor.server_name.len() as u8,
                ],
                descriptor.server_name.as_bytes(),
            )
            .await
    }

    /// Write error to the underlying stream.
    pub async fn write_error(
        &mut self,
        code: ErrorCode,
    ) -> io::Result<()> {
        self.raw_mut()
            .write_all(&[
                Packet::id(PacketType::Error).encode(),
                code as u8,
            ])
            .await
    }

    pub fn raw_mut(&mut self) -> &mut W {
        &mut self.0.raw
    }
}

impl<W: Write, C: Compressor> GalaxyWriter<W, C> {
    // FIXME: Can we offload compression to proxy too?
    pub async fn write_forward(
        &mut self,
        client_id: u16,
        mut buf: &[u8],
        try_compress: bool,
    ) -> io::Result<()> {
        let mut vec = Vec::new();
        let mut flags = PacketFlags::empty();
        if try_compress {
            if let Some(new_buf) = self.compressor.try_compress(buf) {
                vec = new_buf;
                buf = vec.as_slice();
                flags |= PacketFlags::COMPRESSED;
            }
        }

        debug_assert!(if flags.intersects(PacketFlags::COMPRESSED) {
            buf == vec
        } else {
            true
        });

        let (header, header_len) =
            encode_forward_header(client_id, buf.len() as u16, flags);

        self.write_two_bufs(&header[..header_len as usize], buf)
            .await
    }
}

impl<W: Write, C> GalaxyWriter<W, C> {
    async fn write_client_id(
        &mut self,
        id: u16,
        ty: PacketType,
    ) -> io::Result<()> {
        if id <= 0xff {
            self.raw
                .write_all(&[
                    Packet::new(ty, PacketFlags::SHORT_CLIENT).encode(),
                    id as u8,
                ])
                .await
        } else {
            self.raw
                .write_all(&[
                    Packet::id(ty).encode(),
                    (id & 0xff) as u8,
                    (id >> 8) as u8,
                ])
                .await
        }
    }

    #[inline]
    fn write_all<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> impl Future<Output = io::Result<()>> + 'a {
        self.raw.write_all(buf)
    }

    async fn write_two_bufs(
        &mut self,
        mut prepend: &[u8],
        append: &[u8],
    ) -> io::Result<()> {
        let plen = prepend.len();
        let alen = append.len();
        let total = alen + plen;

        if !self.raw.is_write_vectored() {
            // Since write is not vectored we'll just create
            // intermediate buffer
            let mut vec = Vec::with_capacity(total);
            vec.extend(prepend.iter().copied());
            vec.extend(append.iter().copied());

            return self.raw.write_all(&vec).await;
        }

        let mut written = 0_usize;
        let mut ios = [IoSlice::new(prepend), IoSlice::new(append)];
        while written < total {
            let cur_wrote = self.raw.write_vectored(&ios).await?;
            written += cur_wrote;

            if written >= alen && written != total {
                let append_offset = total - written;
                return self.raw.write_all(&append[append_offset..]).await;
            } else if written != total {
                prepend = &prepend[cur_wrote..];
                ios[0] = IoSlice::new(prepend);
            }
        }

        Ok(())
    }
}

impl<W: Write, C> GalaxyWriter<W, C> {
    #[inline]
    pub fn write_disconnected(
        &mut self,
        id: u16,
    ) -> impl Future<Output = io::Result<()>> + '_ {
        self.write_client_id(id, PacketType::Disconnect)
    }
}

impl<W, C> GalaxyWriter<W, C> {
    pub fn into_inner(self) -> (W, C) {
        (self.raw, self.compressor)
    }

    pub fn client(&mut self) -> GalaxyClientWriter<'_, W, C> {
        GalaxyClientWriter(self)
    }

    /// Get server specific packets namespace.
    pub fn server(&mut self) -> GalaxyServerWriter<'_, W, C> {
        GalaxyServerWriter(self)
    }

    pub const fn new(raw: W, compressor: C) -> Self {
        Self { raw, compressor }
    }
}
