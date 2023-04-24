use std::sync::Arc;

use owo_colors::OwoColorize;
use tokio::{
    net::TcpListener,
    sync::mpsc,
};

use crate::{
    config::Config,
    data::commands::http::GlobalHttpCommand,
    features::http::storage::HttpStorage,
    protocols::http::command_handler::handle_command,
};

pub async fn run_http_listener(
    config: Arc<Config>,
    mut rx: mpsc::UnboundedReceiver<GlobalHttpCommand>,
) -> eyre::Result<()> {
    let Some(ref http) = config.http else {
        return Ok(());
    };
    let bold_http = "`HTTP`".bold();
    let discovery_method = http.discovery_method;

    let listener = match TcpListener::bind(http.listen).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!(
                "Failed to bind address {} for the HTTP server: {e}",
                http.listen.bold(),
            );
            return Ok(());
        }
    };
    let bound = listener.local_addr()?;
    let storage = Arc::new(HttpStorage::new());
    let read_buffer = config.server.buffering.read.get();

    tracing::info!("Started {bold_http} server on {}", bound.bold());

    loop {
        let stream;
        let address;

        tokio::select! {
            command = rx.recv() => {
                let Some(command) = command else {
                    tracing::error!(
                        "{bold_http} server failed to pull commands, shutting down protocol server..."
                    );
                    break;
                };

                handle_command(command, &storage).await;
                continue;
            }

            l_result = listener.accept() => {
                (stream, address) = l_result?;
            }
        }

        tracing::info!(
            "{} connected to the {bold_http} server",
            address.bold()
        );
    }

    Ok(())
}
