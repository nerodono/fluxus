use std::{
    future::Future,
    io,
    num::NonZeroU16,
};

use crate::{
    common::write_two_bufs,
    raw::{
        NetworkError,
        PacketType,
        Rights,
    },
    traits::ComposeWrite,
};

pub struct ControlWriter<W> {
    inner: W,
}
pub struct ControlServerWriter<'a, W>(&'a mut W);
pub struct ControlClientWriter<'a, W>(&'a mut W);

impl<'a, W: ComposeWrite> ControlClientWriter<'a, W> {
    pub async fn write_create_http_server(
        &mut self,
        endpoint: Option<&str>,
    ) -> io::Result<()> {
        let endpoint = endpoint.unwrap_or("");
        assert!(endpoint.len() < 255, "endpoint length should be <= u8::MAX");
        write_two_bufs(
            self.0,
            &[PacketType::CreateHttpServer as u8, endpoint.len() as u8],
            endpoint.as_bytes(),
        )
        .await
    }

    pub async fn write_create_tcp_server(
        &mut self,
        port: Option<NonZeroU16>,
    ) -> io::Result<()> {
        let port = port.map_or(0, NonZeroU16::get);
        self.0
            .write_all(&[
                PacketType::CreateTcpServer as u8,
                (port & 0xff) as u8,
                (port >> 8) as u8,
            ])
            .await
    }

    pub async fn write_authorize_password(
        &mut self,
        password: &str,
    ) -> io::Result<()> {
        let psw_len = password.len();
        assert!(
            psw_len < 255,
            "password length must not be greater than u8::MAX"
        );

        write_two_bufs(
            self.0,
            &[PacketType::AuthorizePassword as u8, psw_len as u8],
            password.as_bytes(),
        )
        .await
    }
}

impl<'a, W: ComposeWrite> ControlServerWriter<'a, W> {
    pub async fn write_error(
        &mut self,
        error_code: NetworkError,
    ) -> io::Result<()> {
        self.0
            .write_all(&[PacketType::Error as u8, error_code as u8])
            .await
    }

    pub async fn write_update_rights(
        &mut self,
        new_rights: Rights,
    ) -> io::Result<()> {
        self.0
            .write_all(&[PacketType::UpdateRights as u8, new_rights.bits()])
            .await
    }

    #[inline]
    pub fn write_connected(
        &mut self,
    ) -> impl Future<Output = io::Result<()>> + '_ {
        self.0.write_u8(PacketType::Connected as u8)
    }

    pub async fn write_tcp_server(&mut self, port: u16) -> io::Result<()> {
        self.0
            .write_all(&[
                PacketType::CreateTcpServer as u8,
                (port & 0xff) as u8,
                (port >> 8) as u8,
            ])
            .await
    }

    pub async fn write_hello(
        &mut self,
        connection_id: u16,
        buffer_size: u16,
    ) -> io::Result<()> {
        self.0
            .write_all(&[
                PacketType::Hello as u8,
                (connection_id & 0xff) as u8,
                (connection_id >> 8) as u8,
                (buffer_size & 0xff) as u8,
                (buffer_size >> 8) as u8,
            ])
            .await
    }
}

impl<W> ControlWriter<W> {
    pub fn client(&mut self) -> ControlClientWriter<'_, W> {
        ControlClientWriter(&mut self.inner)
    }

    pub fn server(&mut self) -> ControlServerWriter<'_, W> {
        ControlServerWriter(&mut self.inner)
    }

    pub const fn new(inner: W) -> Self {
        Self { inner }
    }
}
