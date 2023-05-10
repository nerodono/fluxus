use std::{
    borrow::Cow,
    net::SocketAddr,
    num::NonZeroU16,
    sync::Arc,
};

use cfg_if::cfg_if;
use galaxy_network::{
    descriptors::{
        CompressionDescriptor,
        CreateServerResponseDescriptor,
        PingResponseDescriptor,
    },
    raw::{
        ErrorCode,
        PacketFlags,
        Protocol,
        Rights,
    },
    reader::{
        GalaxyReader,
        Read,
    },
    shrinker::interface::{
        Compressor,
        Decompressor,
    },
    writer::{
        GalaxyWriter,
        Write,
    },
};
use owo_colors::OwoColorize;

use crate::{
    config::{
        AuthorizationBackend,
        Config,
    },
    data::{
        proxy::{
            Proxy,
            ProxyData,
        },
        user::User,
    },
    error::{
        NonCriticalError,
        ProcessResult,
    },
    slaves,
    utils::{
        feature_gate::FeatureGate,
        proxy_getter::require_proxy,
        shortcuts::assert_bound,
    },
};

cfg_if! {
    if #[cfg(feature = "tcp")] {
        use tokio::net::TcpListener;
        use crate::servers::tcp::TcpServer;
        use crate::data::commands::tcp::TcpSlaveCommand;
    }
}

cfg_if! {
    if #[cfg(feature = "http")] {
        use crate::servers::http::HttpServer;
        use crate::data::commands::http::{
            HttpServerRequest,
            HttpSlaveCommand
        };
    }
}

pub struct Connection<'a, R, D, W, C> {
    pub reader: GalaxyReader<R, D>,
    pub writer: GalaxyWriter<W, C>,
    pub address: SocketAddr,
    pub user: User,
    pub config: &'a Arc<Config>,
    pub gate: FeatureGate,
}

impl<'a, R, D, W, C> Connection<'a, R, D, W, C>
where
    R: Read,
    W: Write,
    D: Decompressor,
    C: Compressor,
{
    pub async fn server_stopped(&mut self) -> ProcessResult<()> {
        self.writer
            .server()
            .write_error(ErrorCode::ServerStopped)
            .await
            .map_err(Into::into)
    }

    pub async fn forward(&mut self, flags: PacketFlags) -> ProcessResult<()> {
        let client_id = self.reader.read_client_id(flags).await?;
        let length = self.reader.read_forward_length(flags).await? as usize;
        let proxy = match require_proxy(&mut self.user.proxy) {
            Ok(r) => r,
            Err(e) => {
                _ = self.reader.skip_n_bytes::<64>(length).await;
                return Err(e.into());
            }
        };
        let max_read = self.config.server.buffering.read;
        let buffer = self
            .reader
            .try_read_forward_buffer(length, |size| size <= max_read, flags)
            .await?;

        match &mut proxy.data {
            #[cfg(feature = "http")]
            ProxyData::Http(http) => {
                http.channels
                    .send_command(
                        client_id,
                        HttpSlaveCommand::Forward { buf: buffer },
                    )
                    .await?;
            }

            #[cfg(feature = "tcp")]
            ProxyData::Tcp(tcp) => {
                tcp.clients
                    .send_command(
                        client_id,
                        TcpSlaveCommand::Forward { buffer },
                    )
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn disconnected(
        &mut self,
        flags: PacketFlags,
    ) -> ProcessResult<()> {
        let client_id = self.reader.read_client_id(flags).await?;
        let proxy = require_proxy(&mut self.user.proxy)?;

        match &mut proxy.data {
            // Can we get rid of this duplication?
            // Possibly yes...
            #[cfg(feature = "http")]
            ProxyData::Http(http) => {
                let chan = http
                    .channels
                    .remove(client_id)?
                    .return_id(&proxy.pool)
                    .await;
                _ = chan.send(HttpSlaveCommand::Disconnect).await;
            }

            #[cfg(feature = "tcp")]
            ProxyData::Tcp(tcp) => {
                let chan = tcp
                    .clients
                    .remove(client_id)?
                    .return_id(&proxy.pool)
                    .await;
                _ = chan.send(TcpSlaveCommand::Disconnect).await;
            }
        }

        tracing::info!(
            "ID{client_id} was disconnected from the {}'s server",
            self.address.bold()
        );

        Ok(())
    }

    pub async fn create_server(
        &mut self,
        flags: PacketFlags,
    ) -> ProcessResult<()> {
        let protocol = self.reader.read_protocol_type(flags).await?;
        if !self.user.rights.can_create_server(protocol) {
            let skip_no = match protocol {
                Protocol::Tcp | Protocol::Udp => 2,
                Protocol::Http => {
                    todo!();
                }
            };

            _ = self.reader.skip_n_bytes::<16>(skip_no).await;
            return Err(NonCriticalError::NotEnoughRightsForProtocol(
                protocol,
            )
            .into());
        }

        let buffering = &self.config.server.buffering;
        match protocol {
            #[cfg(feature = "tcp")]
            Protocol::Tcp => {
                let port = self.user.select_port(
                    self.reader.read_u16().await?,
                    Protocol::Tcp,
                )?;
                let (bound_to, listener) = assert_bound(
                    port,
                    Protocol::Tcp,
                    TcpListener::bind(("0.0.0.0", port))
                        .await
                        .and_then(|l| Ok((l.local_addr()?, l))),
                )?;
                let server = TcpServer::default();
                let (permit, token, pool) = self.user.replace_proxy(
                    ProxyData::Tcp(server),
                    buffering.channels.get(),
                    Proxy::issue_tcp_permit,
                );
                tokio::spawn(slaves::tcp::listener::listen(
                    self.address,
                    listener,
                    bound_to,
                    permit,
                    pool,
                    buffering.read.get(),
                    buffering.channels,
                    token,
                ));

                self.writer
                    .server()
                    .write_server(&CreateServerResponseDescriptor::Tcp {
                        port: if port == 0 {
                            NonZeroU16::new(bound_to.port())
                        } else {
                            None
                        },
                    })
                    .await?;
                Ok(())
            }

            #[cfg(feature = "http")]
            Protocol::Http => {
                let endpoint = self.reader.read_bytes_prefixed().await?;
                let http_chan = self.gate.http()?;
                if !endpoint.is_empty()
                    && self
                        .user
                        .rights
                        .contains(Rights::CAN_SELECT_DOMAIN)
                {
                    return Err(
                        NonCriticalError::NoAccessToSelectEndpoint.into()
                    );
                }

                let server =
                    HttpServer::new(endpoint.clone(), http_chan.clone());

                // Shutdown tokens can't be monitored reliably, so we just
                // forgot about them. Unbind will be sent by the
                // [`HttpServer`]
                let (permit, _, pool) = self.user.replace_proxy(
                    ProxyData::Http(server),
                    buffering.channels.get(),
                    Proxy::issue_http_permit,
                );
                http_chan.send(HttpServerRequest::Bind {
                    endpoint: Some(endpoint),
                    permit,
                    pool,
                })?;

                // Response will be sent depending on the server response,
                // so client will be notified

                Ok(())
            }

            #[cfg(feature = "udp")]
            Protocol::Udp => {
                todo!();
            }

            #[allow(unreachable_patterns)]
            proto => {
                Err(NonCriticalError::ProtocolIsUnavailable(proto).into())
            }
        }
    }

    pub async fn authorize_password(&mut self) -> ProcessResult<()> {
        match &self.config.authorization {
            AuthorizationBackend::Password { password } => {
                let supplied_password =
                    self.reader.read_string_prefixed().await?;
                if &supplied_password == password {
                    let new_rights =
                        self.config.rights.on_password_auth.to_bits();
                    self.user.rights = new_rights;
                    tracing::info!(
                        "{}'s rights updated: {new_rights:?}",
                        self.address.bold()
                    );
                    self.writer
                        .server()
                        .write_update_rights(new_rights)
                        .await?;
                    Ok(())
                } else {
                    Err(NonCriticalError::IncorrectUniversalPassword.into())
                }
            }
            AuthorizationBackend::Database { .. } => {
                let size = self.reader.read_u8().await? as usize;
                self.reader.skip_n_bytes::<64>(size).await?;
                Err(NonCriticalError::Unimplemented(
                    "authorize through database",
                )
                .into())
            }
        }
    }

    pub async fn ping(&mut self) -> ProcessResult<()> {
        self.writer
            .server()
            .write_ping(&PingResponseDescriptor {
                server_name: Cow::Borrowed(&self.config.server.name),
                buffer_read: self.config.server.buffering.read,
                compression: CompressionDescriptor {
                    algorithm: self.config.compression.algorithm,
                    level: self.config.compression.level,
                },
            })
            .await
            .map_err(Into::into)
    }
}
