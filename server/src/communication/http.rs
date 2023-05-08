use std::num::NonZeroU16;

use galaxy_network::{
    descriptors::CreateServerResponseDescriptor,
    raw::ErrorCode,
    shrinker::interface::Compressor,
    writer::{
        GalaxyWriter,
        Write,
    },
};

use crate::{
    data::{
        commands::http::{
            HttpMasterCommand,
            IdentifiedHttpMasterCommand,
        },
        proxy::Pool,
    },
    error::ProcessResult,
    servers::http::HttpServer,
};

pub async fn handle_command<W, C>(
    pool: &Pool,
    writer: &mut GalaxyWriter<W, C>,
    server: &mut HttpServer,
    threshold: Option<NonZeroU16>,
    IdentifiedHttpMasterCommand { id, command }: IdentifiedHttpMasterCommand,
) -> ProcessResult<bool>
where
    W: Write,
    C: Compressor,
{
    match command {
        HttpMasterCommand::Disconnected => {
            server.channels.remove(id)?.return_id(pool).await;
            writer.write_disconnected(id).await?;
        }

        HttpMasterCommand::Header { buf }
        | HttpMasterCommand::BodyChunk { buf } => {
            let buf_len = buf.len() as u16;
            writer
                .write_forward(
                    id,
                    &buf,
                    threshold.map_or(true, |v| buf_len >= v.get()),
                )
                .await?;
        }

        HttpMasterCommand::FailedToBind => {
            writer
                .server()
                .write_error(ErrorCode::FailedToBindAddress)
                .await?;
        }

        HttpMasterCommand::Connected {
            chan,
            immediate_forward,
        } => {
            let Ok(imm_len): Result<u16, _> = immediate_forward.len().try_into() else {
                // TODO: Display rare error
                return Ok(false);
            };

            server.channels.insert(id, chan);
            _ = writer.server().write_connected(id).await;

            writer
                .write_forward(
                    id,
                    &immediate_forward,
                    threshold.map_or(true, |v| imm_len >= v.get()),
                )
                .await?;
        }

        HttpMasterCommand::Bound { on } => {
            writer
                .server()
                .write_server(&CreateServerResponseDescriptor::Http {
                    endpoint: on,
                })
                .await?;
        }
    }

    Ok(false)
}
