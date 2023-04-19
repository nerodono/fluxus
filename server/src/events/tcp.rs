use std::{
    io,
    net::SocketAddr,
};

use galaxy_network::{
    shrinker::interface::Compressor,
    writer::{
        GalaxyWriter,
        Write,
    },
};
use owo_colors::OwoColorize;

use crate::{
    config::Config,
    data::{
        commands::tcp::TcpMasterCommand,
        servers::tcp::TcpServer,
    },
};

pub async fn handle_tcp_command<W, C>(
    server: &mut TcpServer,
    address: SocketAddr,
    command: TcpMasterCommand,
    writer: &mut GalaxyWriter<W, C>,
    config: &Config,
) -> io::Result<bool>
where
    W: Write,
    C: Compressor,
{
    match command {
        TcpMasterCommand::Forward { id, buffer } => {
            writer
                .write_forward(
                    id,
                    &buffer,
                    buffer.len() >= config.server.buffering.read.get(),
                )
                .await?;
        }

        TcpMasterCommand::Disconnect { id } => {
            writer.write_disconnected(id).await?;
            server.remove_channel(id).await;
        }

        TcpMasterCommand::Connect { id, chan } => {
            server.insert_channel(id, chan);
            writer.server().write_connected(id).await?;
        }

        TcpMasterCommand::Stopped => {
            tracing::error!("{}'s TCP server is stopped", address.bold());
            return Ok(true);
        }
    }

    Ok(false)
}
