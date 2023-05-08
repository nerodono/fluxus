use std::num::NonZeroU16;

use galaxy_network::{
    shrinker::interface::Compressor,
    writer::{
        GalaxyWriter,
        Write,
    },
};

use crate::{
    data::{
        commands::tcp::TcpMasterCommand,
        proxy::Pool,
    },
    error::ProcessResult,
    servers::tcp::TcpServer,
};

pub async fn handle_command<W, C>(
    pool: &Pool,
    writer: &mut GalaxyWriter<W, C>,
    server: &mut TcpServer,
    threshold: Option<NonZeroU16>,
    command: TcpMasterCommand,
) -> ProcessResult<bool>
where
    W: Write,
    C: Compressor,
{
    match command {
        TcpMasterCommand::Forward { id, buffer } => {
            let buf_len = buffer.len();
            writer
                .write_forward(
                    id,
                    &buffer,
                    threshold.map_or(true, |t| buf_len >= (t.get() as usize)),
                )
                .await?;
        }

        TcpMasterCommand::Stopped => {
            return Ok(true);
        }

        TcpMasterCommand::Disconnected { id } => {
            writer.write_disconnected(id).await?;
            server.clients.remove(id)?.return_id(pool).await;
        }

        TcpMasterCommand::Connected { id, chan } => {
            server.clients.insert(id, chan);
            writer.server().write_connected(id).await?;
        }
    }

    Ok(false)
}
