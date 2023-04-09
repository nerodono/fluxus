use std::{
    fmt::Display,
    net::SocketAddr,
};

use galaxy_network::{
    raw::PacketFlags,
    reader::{
        GalaxyReader,
        Read,
        ReadResult,
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
use tokio::sync::mpsc;

use super::{
    server::TcpRemoteServer,
    slave,
};
use crate::tcp::command::SlaveCommand;

pub async fn connect<R: Read, D>(
    buffer_size: usize,
    connect_to: SocketAddr,
    reader: &mut GalaxyReader<R, D>,
    flags: PacketFlags,
    server: &mut TcpRemoteServer,
) -> ReadResult<()> {
    let client_id = reader
        .read_variadic(flags, PacketFlags::SHORT_CLIENT)
        .await?;
    let (chan_tx, chan_rx) = mpsc::unbounded_channel();
    server.new_slave(client_id, chan_tx);

    tokio::spawn(slave::run_slave(
        client_id,
        buffer_size,
        connect_to,
        server.chan_tx.clone(),
        chan_rx,
    ));

    Ok(())
}

pub async fn disconnect<R: Read, D>(
    reader: &mut GalaxyReader<R, D>,
    server: &mut TcpRemoteServer,
    flags: PacketFlags,
) -> ReadResult<()> {
    let client_id = reader
        .read_variadic(flags, PacketFlags::SHORT_CLIENT)
        .await?;
    if let Err(e) = server.remove_client(client_id) {
        tracing::error!(
            "Failed to remove client {client_id} from the local server: \
             {}",
            e.bold()
        );
    } else {
        tracing::info!(
            "Disconnected client {client_id} from the local serer"
        );
    }

    Ok(())
}

pub async fn forward<W, C, R, D>(
    reader: &mut GalaxyReader<R, D>,
    writer: &mut GalaxyWriter<W, C>,
    flags: PacketFlags,
    server: &mut TcpRemoteServer,
) -> ReadResult<()>
where
    W: Write,
    R: Read,
    D: Decompressor,
    C: Compressor,
{
    let client_id = reader
        .read_variadic(flags, PacketFlags::SHORT_CLIENT)
        .await?;
    let length = reader
        .read_variadic(flags, PacketFlags::SHORT)
        .await? as usize;
    let buffer = if flags.intersects(PacketFlags::COMPRESSED) {
        reader
            .try_read_compressed(length, |_| true)
            .await?
    } else {
        reader.read_buffer(length).await?
    };

    if let Err(e) =
        server.send_command(client_id, SlaveCommand::Forward { buffer })
    {
        return fallback_disconnect(
            server, client_id, "forward", e, writer,
        )
        .await;
    }
    Ok(())
}

async fn fallback_disconnect<W: Write, C>(
    server: &mut TcpRemoteServer,
    id: u16,
    fn_: impl Display,
    error: impl Display,
    writer: &mut GalaxyWriter<W, C>,
) -> ReadResult<()> {
    server.just_remove_client(id);
    writer.write_disconnected(id).await?;
    tracing::error!(
        "Failed to send command to the client with ID {id}: {error} \
         ({fn_})"
    );
    Ok(())
}
