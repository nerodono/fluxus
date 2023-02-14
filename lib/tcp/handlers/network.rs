use std::{
    io,
    net::SocketAddr,
};

use mid_net::{
    prelude::*,
    proto::{
        PacketType,
        Protocol,
        ProtocolError,
    },
    utils::flags,
};

use crate::{
    config::base::{
        Config,
        ProtocolPermissionsCfg,
    },
    tcp::state::{
        Permissions,
        State,
    },
};

/// This is called when `create server` packet issued.
/// Creates server for supplied protocol.
pub async fn on_create_server<W, R, C, D>(
    writer: &mut MidWriter<W, C>,
    reader: &mut MidReader<R, D>,
    packet_flags: u8,
) -> io::Result<()>
where
    W: WriterUnderlyingExt,
    R: ReaderUnderlyingExt,
{
    let protocol = if flags::is_compressed(packet_flags) {
        Protocol::Tcp
    } else {
        Protocol::Udp
    };

    match protocol {
        Protocol::Tcp => {
            let port = if flags::is_compressed(packet_flags) {
                0
            } else {
                reader.read_u16().await?
            };
            todo!()
        }
        Protocol::Udp => {
            writer
                .server()
                .write_failure(ProtocolError::Unimplemented)
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
    from: SocketAddr,
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
    from: SocketAddr,
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
