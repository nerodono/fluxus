use std::{
    io,
    net::SocketAddr,
    sync::Arc,
};

use mid_net::{
    prelude::{
        impl_::interface::{
            ICompressor,
            IDecompressor,
        },
        *,
    },
    proto::{
        PacketType,
        Protocol,
        ProtocolError,
    },
    utils::flags,
};
use tokio::net::TcpListener;

use super::{
    message::SlaveMessage,
    utils::send_slave_message_to,
};
use crate::{
    config::base::{
        Config,
        ProtocolPermissionsCfg,
    },
    tcp::{
        slave::listener,
        state::{
            Permissions,
            State,
        },
        views::MasterStateView,
    },
};

/// Triggered when `forward` packet arrived.
pub async fn on_forward<W, R, C, D>(
    writer: &mut MidWriter<W, C>,
    reader: &mut MidReader<R, D>,
    state: &State,
    from: &SocketAddr,
    flags: u8,
    constraint: DecompressionConstraint,
) -> io::Result<()>
where
    W: WriterUnderlyingExt,
    R: ReaderUnderlyingExt,
    C: ICompressor,
    D: IDecompressor,
{
    match state.server {
        Some(ref server) => {
            // TODO: restrict maximum length
            let client_id = reader.read_client_id(flags).await?;
            let length = reader.read_length(flags).await?;
            let buffer = if flags::is_compressed(flags) {
                reader
                    .read_compressed(
                        length as usize,
                        DecompressionStrategy::ConstrainedConst { constraint },
                    )
                    .await
            } else {
                reader
                    .read_buffer(length as usize)
                    .await
                    .map_err(|e| e.into())
            };
            let buffer = match buffer {
                Ok(b) => b,
                Err(CompressedReadError::Io(error)) => return Err(error),
                Err(e) => {
                    tracing::error!(
                        %from,
                        "Failed to decompress forward packet: {e}"
                    );
                    return Ok(());
                }
            };

            if server.forward(client_id, buffer).await.is_err() {
                writer
                    .server()
                    .write_failure(ProtocolError::ClientDoesNotExists)
                    .await
            } else {
                Ok(())
            }
        }

        None => {
            writer
                .server()
                .write_failure(ProtocolError::ServerIsNotCreated)
                .await
        }
    }
}

/// Called when `disconnected` packet arrived
pub async fn on_disconnect<W, R, C, D>(
    writer: &mut MidWriter<W, C>,
    reader: &mut MidReader<R, D>,
    state: &mut State,
    flags: u8,
) -> io::Result<()>
where
    W: WriterUnderlyingExt,
    R: ReaderUnderlyingExt,
{
    let client_id = reader.read_client_id(flags).await?;
    send_slave_message_to(writer, client_id, state, SlaveMessage::Disconnect)
        .await?;
    Ok(())
}

/// This is called when `create server` packet issued.
/// Creates server for supplied protocol.
pub async fn on_create_server<W, R, C, D>(
    writer: &mut MidWriter<W, C>,
    reader: &mut MidReader<R, D>,
    state: &mut State,
    from: &SocketAddr,
    packet_flags: u8,
) -> io::Result<()>
where
    W: WriterUnderlyingExt,
    R: ReaderUnderlyingExt,
{
    if state.has_server() {
        return writer
            .server()
            .write_failure(ProtocolError::AlreadyCreated)
            .await;
    }

    let protocol = if flags::is_compressed(packet_flags) {
        Protocol::Tcp
    } else {
        Protocol::Udp
    };

    match protocol {
        Protocol::Tcp if state.permissions.can(Permissions::CREATE_TCP) => {
            let port = if flags::is_compressed(packet_flags) {
                0
            } else {
                let port = reader.read_u16().await?;
                if state
                    .permissions
                    .can(Permissions::SELECT_TCP_PORT)
                {
                    port
                } else {
                    tracing::error!(
                        %from,
                        port,
                        "Create server with custom port failed: access denied"
                    );
                    return writer
                        .server()
                        .write_failure(ProtocolError::AccessDenied)
                        .await;
                }
            };
            let listener = match TcpListener::bind(("0.0.0.0", port)).await {
                Ok(l) => l,
                Err(error) => {
                    tracing::error!(
                        %error,
                        %from,
                        "Failed to create TCP listener"
                    );

                    return writer
                        .server()
                        .write_failure(ProtocolError::FailedToCreateListener)
                        .await;
                }
            };
            let listening_at_port = if port == 0 {
                match listener.local_addr().map(|a| a.port()) {
                    Ok(p) => p,
                    Err(error) => {
                        tracing::error!(
                            %from,
                            %error,
                            "Failed to retrieve TCP port from the system"
                        );

                        return writer
                            .server()
                            .write_failure(ProtocolError::FailedToRetrievePort)
                            .await;
                    }
                }
            } else {
                port
            };

            let (shutdown_token, master_tx, created_server) =
                state.create_server(listening_at_port);
            tracing::info!(%from, "Started server at 0.0.0.0:{listening_at_port}");

            tokio::spawn(listener::run_slave_tcp_listener(
                listener,
                *from,
                shutdown_token,
                MasterStateView {
                    pool: Arc::clone(&created_server.pool),
                    master: master_tx,
                },
            ));

            writer
                .server()
                .write_server(listening_at_port)
                .await
        }

        Protocol::Udp if state.permissions.can(Permissions::CREATE_UDP) => {
            writer
                .server()
                .write_failure(ProtocolError::Unimplemented)
                .await
        }

        tried_proto => {
            tracing::error!(
                %from,
                ?tried_proto,
                "Create server with custom protocol failed: access denied"
            );
            writer
                .server()
                .write_failure(ProtocolError::AccessDenied)
                .await
        }
    }
}

/// Tries to authorize user using supplied password. On
/// success changes its permissions to the
/// `universal_password` permissions level.
pub async fn on_authorize<W, R, C, D>(
    writer: &mut MidWriter<W, C>,
    reader: &mut MidReader<R, D>,
    state: &mut State,
    from: &SocketAddr,
    success_perms: &ProtocolPermissionsCfg,
    actual_password: &Option<String>,
) -> io::Result<()>
where
    W: WriterUnderlyingExt,
    R: ReaderUnderlyingExt,
{
    let supplied_password = reader.read_string_prefixed().await?;
    if let Some(actual_password) = actual_password {
        if &supplied_password == actual_password {
            state.permissions = Permissions::from_cfg(success_perms);
            tracing::info!(
                %from,
                supplied_password,
                "Universal password authorization request: access granted"
            );
            writer
                .server()
                .write_update_rights(state.permissions.bits())
                .await
        } else {
            tracing::error!(
                %from,
                supplied_password,
                "Universal password authorization request: wrong password"
            );
            writer
                .server()
                .write_failure(ProtocolError::AccessDenied)
                .await
        }
    } else {
        tracing::error!(
            %from,
            supplied_password,
            "Universal password authorization request: feature is disabled"
        );

        writer
            .server()
            .write_failure(ProtocolError::Disabled)
            .await
    }
}

/// Reacts to the `ping`. Basically writes the server name,
/// compression algorithm and the read bufferization
/// settings
pub async fn on_ping<W: WriterUnderlyingExt, C>(
    writer: &mut MidWriter<W, C>,
    config: &Config,
) -> io::Result<()> {
    writer
        .server()
        .write_ping(
            &config.server.name,
            config.compression.tcp.algorithm,
            config
                .server
                .bufferization
                .read
                .try_into()
                .unwrap_or_else(|e| {
                    let fallback_maximum = u16::MAX;
                    tracing::error!(
                        fallback_maximum,
                        "Failed to write bufferization value ({e}), writing \
                         back fallback maximum"
                    );

                    fallback_maximum
                }),
        )
        .await
}

/// Triggered if packet type is unexpected for the server
/// side, for example: `error` packet is unexpected.
pub async fn on_unexpected<W: WriterUnderlyingExt, C>(
    writer: &mut MidWriter<W, C>,
    from: &SocketAddr,
    packet_type: PacketType,
) -> io::Result<()> {
    tracing::error!(?packet_type, %from, "Sent unexpected packet");
    writer
        .server()
        .write_failure(ProtocolError::UnexpectedPacket)
        .await
}

/// Called when router receives unknown packet type.
/// Basically just logs & writes the error
pub async fn on_unknown_packet<W: WriterUnderlyingExt, C>(
    writer: &mut MidWriter<W, C>,
    from: SocketAddr,
    packet_type: u8,
    packet_flags: u8,
) -> io::Result<()> {
    tracing::error!(
        packet_type,
        packet_flags,
        %from,
        "Unknown packet type received"
    );
    writer
        .server()
        .write_failure(ProtocolError::UnknownPacket)
        .await
}
