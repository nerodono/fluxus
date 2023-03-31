use std::{
    future::Future,
    io::{
        self,
        IoSlice,
    },
    num::NonZeroU16,
};

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
    },
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

impl<W: Write, C> GalaxyServerWriter<'_, W, C> {
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

impl<W: Write, C> GalaxyWriter<W, C> {
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

impl<W, C> GalaxyWriter<W, C> {
    /// Get server specific packets namespace.
    pub fn server(&mut self) -> GalaxyServerWriter<'_, W, C> {
        GalaxyServerWriter(self)
    }

    pub const fn new(raw: W, compressor: C) -> Self {
        Self { raw, compressor }
    }
}
