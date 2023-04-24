use std::{
    borrow::Cow,
    net::SocketAddr,
    num::NonZeroU16,
};

use cfg_if::cfg_if;
use galaxy_network::{
    descriptors::{
        CompressionDescriptor,
        CreateServerResponseDescriptor,
        PingResponseDescriptor,
    },
    error::ReadError,
    raw::{
        ErrorCode,
        PacketFlags,
        Protocol,
    },
    reader::{
        GalaxyReader,
        Read,
    },
    shrinker::interface::Decompressor,
    writer::{
        GalaxyWriter,
        Write,
    },
};
use owo_colors::OwoColorize;
use tokio::net::TcpListener;

cfg_if! {
    if #[cfg(feature = "http")] {
        use crate::config::HttpDiscoveryMethod;
        use crate::data::servers::http::HttpServer;
        use crate::data::commands::http::GlobalHttpCommand;
    }
}

use crate::{
    config::{
        AuthorizationBackend,
        Config,
    },
    data::{
        commands::tcp::TcpSlaveCommand,
        id_pool::IdPoolImpl,
        proxy::{
            ProxyData,
            ServingProxy,
        },
        servers::tcp::TcpServer,
        user::User,
    },
    error::ProcessResult,
    slaves::tcp::listener::tcp_slave_listen,
    utils::{
        compiler::unlikely,
        feature_gate::FeatureGate,
        proxy_shortcuts::{
            require_proxy,
            treat_send_result,
        },
    },
};

pub async fn authorize_password<R, D, W, C>(
    reader: &mut GalaxyReader<R, D>,
    writer: &mut GalaxyWriter<W, C>,
    user: &mut User,
    config: &Config,
) -> ProcessResult<()>
where
    W: Write,
    R: Read,
{
    let supplied_password = reader.read_string_prefixed().await?;
    match &config.authorization {
        AuthorizationBackend::Password { password } => {
            if password == &supplied_password {
                user.rights = config.rights.on_password_auth.to_bits();
                writer
                    .server()
                    .write_update_rights(user.rights)
                    .await?;
            } else {
                writer
                    .server()
                    .write_error(ErrorCode::AccessDenied)
                    .await?;
            }
        }

        AuthorizationBackend::Database { .. } => {
            writer
                .server()
                .write_error(ErrorCode::Unsupported)
                .await?;
        }
    }

    Ok(())
}

pub async fn disconnect<R: Read, D>(
    reader: &mut GalaxyReader<R, D>,
    user: &mut User,
    flags: PacketFlags,
) -> ProcessResult<()> {
    let client_id = reader.read_client_id(flags).await?;
    let proxy = require_proxy(&mut user.proxy)?;

    match &mut proxy.data {
        ProxyData::Tcp(tcp) => {
            treat_send_result(
                tcp.send_command(client_id, TcpSlaveCommand::Disconnect),
            )?;
        }

        #[cfg(feature = "http")]
        ProxyData::Http(http) => {
            todo!();
        }
    }

    Ok(())
}

pub async fn forward<R, D>(
    reader: &mut GalaxyReader<R, D>,
    user: &mut User,
    flags: PacketFlags,
    config: &Config,
) -> ProcessResult<()>
where
    R: Read,
    D: Decompressor,
{
    let max_read = config.server.buffering.read.get();

    let client_id = reader.read_client_id(flags).await?;
    let length = reader.read_forward_length(flags).await? as usize;

    if unlikely(length >= max_read) {
        _ = reader.skip_n_bytes::<128>(length).await;
        return Err(ReadError::TooLongBuffer.into());
    }

    let buffer = reader
        .try_read_forward_buffer(length, |size| size.get() <= max_read, flags)
        .await?;
    let proxy = require_proxy(&mut user.proxy)?;
    match &mut proxy.data {
        ProxyData::Tcp(tcp) => {
            treat_send_result(tcp.send_command(
                client_id,
                TcpSlaveCommand::Forward { buffer },
            ))?;
        }

        #[cfg(feature = "http")]
        ProxyData::Http(http) => {
            todo!();
        }
    }
    Ok(())
}

pub async fn create_server<R, D, W, C>(
    reader: &mut GalaxyReader<R, D>,
    writer: &mut GalaxyWriter<W, C>,
    user: &mut User,
    address: SocketAddr,
    flags: PacketFlags,
    config: &Config,
    id_pool_factory: impl Fn() -> IdPoolImpl,
    #[allow(unused)] gate: &FeatureGate,
) -> ProcessResult<()>
where
    W: Write,
    R: Read,
{
    let protocol = reader.read_protocol_type(flags).await?;

    match protocol {
        #[cfg(feature = "http")]
        Protocol::Http => {
            let discovery_data = reader.read_bytes_prefixed().await?;
            let Some(ref http_cfg) = config.http else {
                _ = writer.server().write_error(ErrorCode::Unsupported).await;
                return Ok(());
            };
            let feature = gate.http()?;

            // TODO: permission controlling
            match http_cfg.discovery_method {
                HttpDiscoveryMethod::Path => {
                    let pool = id_pool_factory();
                    let http_server = HttpServer::new(
                        discovery_data.clone(),
                        feature.clone(),
                    );
                    let (server, _) =
                        ServingProxy::new(ProxyData::Http(http_server));
                    let permit = server.issue_http_permit().unwrap();
                    user.proxy = Some(server);

                    feature.send_command(GlobalHttpCommand::Bind {
                        to: Some(discovery_data),
                        permit,
                        pool,
                    })?;
                }
            }
        }

        Protocol::Tcp => {
            let port = reader.read_u16().await?;
            if port != 0 && !user.rights.can_select_port(Protocol::Tcp) {
                writer
                    .server()
                    .write_error(ErrorCode::AccessDenied)
                    .await?;
                return Ok(());
            }

            let (bound_to, listener) =
                match TcpListener::bind(("0.0.0.0", port))
                    .await
                    .and_then(|l| Ok((l.local_addr()?, l)))
                {
                    Ok(l) => l,
                    Err(e) => {
                        tracing::error!(
                            "{} failed to bind {}: {e}",
                            address.bold(),
                            format_args!("0.0.0.0:{port}").bold()
                        );
                        writer
                            .server()
                            .write_error(ErrorCode::FailedToBindAddress)
                            .await?;
                        return Ok(());
                    }
                };
            tracing::info!(
                "{} created TCP server on {}",
                address.bold(),
                bound_to.bold()
            );

            let pool = id_pool_factory();
            let (created_proxy, shutdown_token) = ServingProxy::new(
                ProxyData::Tcp(TcpServer::new(pool.clone())),
            );
            let permit = created_proxy.issue_tcp_permit().unwrap();

            tokio::spawn(tcp_slave_listen(
                permit,
                pool,
                shutdown_token,
                listener,
                address,
                config.server.buffering.read.get(),
            ));
            user.proxy = Some(created_proxy);

            writer
                .server()
                .write_server(&CreateServerResponseDescriptor::Tcp {
                    port: if port == 0 {
                        NonZeroU16::new(bound_to.port())
                    } else {
                        None
                    },
                })
                .await?;
        }

        #[allow(unreachable_patterns)]
        p => {
            tracing::info!(
                "{} server was compiled without `{p:?}` support",
                address.bold()
            );
            writer
                .server()
                .write_error(ErrorCode::Unimplemented)
                .await?;
        }
    }

    Ok(())
}

pub async fn ping<W: Write, C>(
    writer: &mut GalaxyWriter<W, C>,
    config: &Config,
) -> ProcessResult<()> {
    writer
        .server()
        .write_ping(&PingResponseDescriptor {
            server_name: Cow::Borrowed(&config.server.name),
            buffer_read: config.server.buffering.read,
            compression: CompressionDescriptor {
                level: config.compression.level,
                algorithm: config.compression.algorithm,
            },
        })
        .await?;
    Ok(())
}
