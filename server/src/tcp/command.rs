use std::io;

use galaxy_network::{
    shrinker::interface::Compressor,
    writer::{
        GalaxyWriter,
        Write,
    },
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    config::Config,
    logic::{
        command::SlaveCommand,
        tcp_server::TcpProxyServer,
    },
};

pub async fn forward<W: Write, C: Compressor>(
    writer: &mut GalaxyWriter<W, C>,
    id: u16,
    buffer: Vec<u8>,
    config: &Config,
) -> io::Result<()> {
    writer
        .write_forward(
            id,
            &buffer,
            match config.compression.threshold {
                Some(threshold) => buffer.len() as u16 >= threshold.get(),

                None => true,
            },
        )
        .await?;
    Ok(())
}

pub async fn disconnected<W: Write, C>(
    writer: &mut GalaxyWriter<W, C>,
    server: &mut TcpProxyServer,
    id: u16,
) -> io::Result<()> {
    let _ = server.unmap_client(id);
    writer.write_disconnected(id).await
}

pub async fn connected<W: Write, C>(
    writer: &mut GalaxyWriter<W, C>,
    server: &mut TcpProxyServer,
    id: u16,
    chan: UnboundedSender<SlaveCommand>,
) -> io::Result<()> {
    server.map_client(id, chan);
    writer.server().write_connected(id).await
}
