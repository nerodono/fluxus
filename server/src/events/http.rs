use std::{
    io,
    net::SocketAddr,
};

use galaxy_network::{
    descriptors::CreateServerResponseDescriptor,
    raw::ErrorCode,
    writer::{
        GalaxyWriter,
        Write,
    },
};
use owo_colors::OwoColorize;

use crate::{
    config::Config,
    data::{
        commands::http::HttpMasterCommand,
        servers::http::HttpServer,
    },
};

pub async fn handle_http_command<W: Write, C>(
    _server: &mut HttpServer,
    address: SocketAddr,
    command: HttpMasterCommand,
    writer: &mut GalaxyWriter<W, C>,
    _config: &Config,
) -> io::Result<bool> {
    match command {
        HttpMasterCommand::BoundEndpoint { on } => {
            writer
                .server()
                .write_server(&CreateServerResponseDescriptor::Http {
                    endpoint: on,
                })
                .await?;
        }

        HttpMasterCommand::FailedToBindEndpoint { error } => {
            tracing::error!(
                "{} failed to create HTTP endpoint: {error}",
                address.bold()
            );

            writer
                .server()
                .write_error(ErrorCode::FailedToBindAddress)
                .await?;
        }
    }

    Ok(false)
}
