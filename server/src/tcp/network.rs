use std::{
    fmt::Display,
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
    raw::{
        ErrorCode,
        Packet,
        Protocol,
    },
    reader::{
        GalaxyReader,
        Read,
        ReadResult,
    },
    writer::{
        GalaxyWriter,
        Write,
    },
};
use owo_colors::OwoColorize;
use tokio::net::TcpListener;

use crate::{
    config::Config,
    logic::{
        tcp_server::{
            TcpIdPool,
            TcpProxyServer,
        },
        user::User,
    },
    tcp::proxy,
};

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
