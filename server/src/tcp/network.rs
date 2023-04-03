use std::{
    fmt::Display,
    io,
    net::SocketAddr,
    num::NonZeroU16,
    sync::Arc,
};

use galaxy_network::{
    descriptors::{
        CompressionDescriptor,
        CreateServerResponseDescriptor,
        PingResponseDescriptor,
    },
    error::ReadError,
    raw::{
        ErrorCode,
        Packet,
        PacketFlags,
        Protocol,
    },
    reader::{
        GalaxyReader,
        Read,
        ReadResult,
    },
    shrinker::interface::Decompressor,
    writer::{
        GalaxyWriter,
        Write,
    },
};
use owo_colors::OwoColorize;
use tokio::net::TcpListener;

use crate::{
    config::{
        AuthorizationBackend,
        Config,
    },
    error::ChanSendError,
    logic::{
        command::SlaveCommand,
        tcp_server::{
            TcpIdPool,
            TcpProxyServer,
        },
        user::User,
    },
    tcp::proxy,
};

pub async fn authorize_password<W, C, R, D>(
    address: SocketAddr,
    writer: &mut GalaxyWriter<W, C>,
    reader: &mut GalaxyReader<R, D>,
    user: &mut User,
    config: &Config,
) -> io::Result<()>
where
    W: Write,
    R: Read,
{
    let password = reader.read_string_prefixed().await?;
    match config.authorization {
        AuthorizationBackend::Password {
            password: ref actual_pswd,
        } => {
            if actual_pswd.as_str() != password {
                tracing::error!(
                    "{} Aailed to authorize using universal password: \
                     wrong password",
                    address.bold()
                );
                writer
                    .server()
                    .write_error(ErrorCode::AccessDenied)
                    .await
            } else {
                let rights = config.rights.on_password_auth.to_bits();
                tracing::info!(
                    "{} Authorized through the universal password and \
                     got the following rights: {:?}",
                    address.bold(),
                    rights.green()
                );
                user.rights = rights;
                writer.server().write_update_rights(rights).await
            }
        }

        AuthorizationBackend::Database { .. } => {
            tracing::error!(
                "{} Tried to authorize using universal password: disabled",
                address.bold()
            );
            writer
                .server()
                .write_error(ErrorCode::Unsupported)
                .await
        }
    }
}

pub async fn forward<W, C, R, D>(
    address: SocketAddr,
    writer: &mut GalaxyWriter<W, C>,
    reader: &mut GalaxyReader<R, D>,
    pkt: Packet,
    user: &mut User,
    config: &Config,
    running: &mut bool,
) -> ReadResult<()>
where
    W: Write,
    R: Read,
    D: Decompressor,
{
    let id = reader
        .read_variadic(pkt.flags, PacketFlags::SHORT_CLIENT)
        .await?;
    let length = reader
        .read_variadic(pkt.flags, PacketFlags::SHORT)
        .await? as usize;
    let max_length = config.server.buffering.read.get();

    if length > max_length {
        return stop_server_with_error(
            address,
            writer,
            ErrorCode::TooLongBuffer,
            running,
        )
        .await;
    }

    let buffer = match reader
        .try_read_compressed(length, |len| len.get() <= max_length)
        .await
    {
        Ok(buf) => buf,
        Err(ReadError::FailedToDecompress) => {
            tracing::error!(
                "{} Failed to decompress buffer of size {length}bytes",
                address.bold()
            );
            *running = false;
            writer
                .server()
                .write_error(ErrorCode::FailedToDecompress)
                .await?;
            return Ok(());
        }

        Err(e) => return Err(e),
    };

    send_tcp_command(
        writer,
        id,
        &mut user.tcp_proxy,
        SlaveCommand::Forward { buffer },
        address,
        "forward",
    )
    .await?;

    Ok(())
}

pub async fn disconnect<W, C, R, D>(
    address: SocketAddr,
    writer: &mut GalaxyWriter<W, C>,
    reader: &mut GalaxyReader<R, D>,
    pkt: Packet,
    user: &mut User,
) -> io::Result<()>
where
    W: Write,
    R: Read,
{
    let id = reader
        .read_variadic(pkt.flags, PacketFlags::SHORT_CLIENT)
        .await?;

    let Some(server) = send_tcp_command(
        writer,
        id,
        &mut user.tcp_proxy,
        SlaveCommand::Disconnect,
        address,
        "disconnect",
    )
    .await? else {
        return Ok(());
    };

    expect("disconnect", server.unmap_client(id))
}

pub async fn server_request<W, C, R, D>(
    address: SocketAddr,
    writer: &mut GalaxyWriter<W, C>,
    reader: &mut GalaxyReader<R, D>,
    pkt: Packet,
    user: &mut User,
    id_pool_factory: &mut (impl Send + FnMut() -> TcpIdPool),
) -> ReadResult<()>
where
    W: Write,
    R: Read,
{
    let protocol = reader.read_protocol_type(pkt.flags).await?;
    let port = reader.read_u16().await?;

    if !user.can_create_server(protocol) {
        return access_denied(
            address,
            // FIXME: Unnecessary allocation
            format!("Create {protocol:?} server"),
            writer,
        )
        .await;
    } else if port != 0 && !user.can_select_port(protocol) {
        return access_denied(
            address,
            // FIXME: Can it be replaced? Unnecessary allocation
            format!("Select {protocol:?} port"),
            writer,
        )
        .await;
    }

    match protocol {
        Protocol::Tcp => {
            let (listener, bound_address) =
                match TcpListener::bind(("0.0.0.0", port))
                    .await
                    .and_then(|listener| {
                        let address = listener.local_addr()?;
                        Ok((listener, address))
                    }) {
                    Ok(l) => l,
                    Err(e) => {
                        tracing::error!(
                            "Failed to bind {} for {}: {e}",
                            format_args!("0.0.0.0:{port}").bold(),
                            address.bold()
                        );
                        return writer
                            .server()
                            .write_error(ErrorCode::FailedToBindAddress)
                            .await
                            .map_err(Into::into);
                    }
                };
            tracing::info!(
                "Bound TCP server on {} for {}",
                bound_address.bold(),
                address.bold()
            );

            let (tcp_server, token) = TcpProxyServer::new(
                bound_address,
                address,
                id_pool_factory(),
            );

            tokio::spawn(proxy::listen_tcp_proxy(
                address,
                Arc::clone(&tcp_server.pool),
                listener,
                token,
                tcp_server.send_chan.clone(),
            ));
            user.tcp_proxy = Some(tcp_server);

            writer
                .server()
                .write_server(&CreateServerResponseDescriptor {
                    port: if port == 0 {
                        NonZeroU16::new(bound_address.port())
                    } else {
                        None
                    },
                })
                .await?;

            Ok(())
        }

        proto => {
            unimplemented(
                address,
                // FIXME: Unnecessary allocation
                format!("requested protocol {proto:?}"),
                writer,
            )
            .await
        }
    }
}

pub async fn ping<W: Write, C>(
    writer: &mut GalaxyWriter<W, C>,
    config: &Config,
) -> ReadResult<()> {
    writer
        .server()
        .write_ping(&PingResponseDescriptor {
            compression: CompressionDescriptor {
                algorithm: config.compression.algorithm,
                level: config.compression.level,
            },

            server_name: &config.server.name,
            buffer_read: config.server.buffering.read,
        })
        .await
        .map_err(Into::into)
}

//

async fn stop_server_with_error<W: Write, C>(
    address: SocketAddr,
    writer: &mut GalaxyWriter<W, C>,
    error: ErrorCode,
    running: &mut bool,
) -> ReadResult<()> {
    tracing::error!(
        "Stopped serving of {} due to the following error: {error}",
        address.bold()
    );
    *running = false;
    writer
        .server()
        .write_error(error)
        .await
        .map_err(Into::into)
}

#[inline]
fn expect<E>(fn_name: &str, r: Result<(), E>) -> io::Result<()> {
    r.unwrap_or_else(|_| {
        panic!(
            "This behavior of {fn_name} is unexpected, report about it \
             on github"
        )
    });
    Ok(())
}

async fn send_tcp_command<'a, W: Write, C>(
    writer: &mut GalaxyWriter<W, C>,
    id: u16,
    proxy: &'a mut Option<TcpProxyServer>,
    command: SlaveCommand,
    address: SocketAddr,
    action_name: impl Display,
) -> io::Result<Option<&'a mut TcpProxyServer>> {
    match proxy {
        Some(ref mut server) => {
            match server.send_to(id, command) {
                Ok(()) => Ok(Some(server)),

                // Do not report this kind of error
                Err(ChanSendError::ChannelIsClosed) => Ok(None),
                Err(ChanSendError::IdDoesNotExists) => {
                    tracing::error!(
                        "{} ID not found ({action_name})",
                        address.bold()
                    );
                    writer
                        .server()
                        .write_error(ErrorCode::ClientDoesNotExists)
                        .await?;
                    Ok(None)
                }
            }
        }

        None => {
            tracing::error!(
                "{} hasn't created the server ({action_name})",
                address.bold()
            );
            writer
                .server()
                .write_error(ErrorCode::ServerWasNotCreated)
                .await?;
            Ok(None)
        }
    }
}

async fn write_error<W: Write, C>(
    address: SocketAddr,
    str_error: &str,
    action: impl Display,
    error: ErrorCode,
    writer: &mut GalaxyWriter<W, C>,
) -> ReadResult<()> {
    tracing::error!("{} {}: {}", address.bold(), str_error, action);
    writer
        .server()
        .write_error(error)
        .await
        .map_err(Into::into)
}

async fn access_denied<W: Write, C>(
    address: SocketAddr,
    action: impl Display,
    writer: &mut GalaxyWriter<W, C>,
) -> ReadResult<()> {
    write_error(
        address,
        "Access denied",
        action,
        ErrorCode::AccessDenied,
        writer,
    )
    .await
}

async fn unimplemented<W: Write, C>(
    address: SocketAddr,
    action: impl Display,
    writer: &mut GalaxyWriter<W, C>,
) -> ReadResult<()> {
    write_error(
        address,
        "Unimplemented",
        action,
        ErrorCode::Unimplemented,
        writer,
    )
    .await
}
