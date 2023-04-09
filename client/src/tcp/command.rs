use std::io;

use galaxy_network::{
    shrinker::interface::Compressor,
    writer::{
        GalaxyWriter,
        Write,
    },
};

use super::server::TcpRemoteServer;

pub enum MasterCommand {
    Forward { id: u16, buffer: Vec<u8> },
    Disconnect { id: u16 },
}

pub enum SlaveCommand {
    Forward { buffer: Vec<u8> },
    Disconnect,
}

#[inline]
pub async fn disconnect<W: Write, C>(
    writer: &mut GalaxyWriter<W, C>,
    server: &mut TcpRemoteServer,
    id: u16,
) -> io::Result<()> {
    writer.write_disconnected(id).await?;
    server.just_remove_client(id);
    tracing::info!("Client {id} disconnected from the local server");
    Ok(())
}

#[inline]
pub async fn forward<W, C>(
    writer: &mut GalaxyWriter<W, C>,
    id: u16,
    buffer: Vec<u8>,
) -> io::Result<()>
where
    W: Write,
    C: Compressor,
{
    writer
        .write_forward(id, &buffer, buffer.len() >= 80)
        .await?;
    Ok(())
}
