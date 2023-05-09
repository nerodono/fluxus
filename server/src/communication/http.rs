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
    _threshold: Option<NonZeroU16>,
    IdentifiedHttpMasterCommand { id, command }: IdentifiedHttpMasterCommand,
) -> ProcessResult<bool>
where
    W: Write,
    C: Compressor,
{
    match command {
        HttpMasterCommand::Disconnected => {
            if let Ok(r) = server.channels.remove(id) {
                r.return_id(pool).await;
            }
            writer.write_disconnected(id).await?;
        }

        HttpMasterCommand::Forward { buffer } => {
            writer.write_forward(id, &buffer, false).await?;
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
            let Ok(_imm_len): Result<u16, _> = immediate_forward.len().try_into() else {
                // TODO: Display rare error
                tracing::error!("TODO: Error");
                return Ok(false);
            };

            server.channels.insert(id, chan);
            _ = writer.server().write_connected(id).await;

            writer
                .write_forward(id, &immediate_forward, false)
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
