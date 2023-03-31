use std::{
    fmt::Display,
    net::SocketAddr,
};

use galaxy_network::{
    descriptors::{
        CompressionDescriptor,
        PingResponseDescriptor,
    },
    raw::{
        ErrorCode,
        Packet,
        Protocol,
        Rights,
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

use crate::{
    config::Config,
    logic::user::User,
};

pub async fn server_request<W, C, R, D>(
    address: SocketAddr,
    writer: &mut GalaxyWriter<W, C>,
    reader: &mut GalaxyReader<R, D>,
    pkt: Packet,
    user: &mut User,
) -> ReadResult<()>
where
    W: Write,
    R: Read,
{
    let protocol = reader.read_protocol_type(pkt.flags).await?;
    let port = reader.read_u16().await?;

    if !user.rights.contains(Rights::CAN_CREATE_TCP) {
        return access_denied(address, "create TCP server", writer).await;
    }
    if port != 0 && !user.rights.contains(Rights::CAN_SELECT_TCP_PORT) {
        return access_denied(address, "select TCP port", writer).await;
    }

    match protocol {
        Protocol::Tcp => {
            todo!()
        }

        proto => {
            unimplemented(
                address,
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

async fn write_error<W: Write, C>(
    address: SocketAddr,
    str_error: &str,
    action: impl Display + Send,
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
    action: impl Display + Send,
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
    action: impl Display + Send,
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
