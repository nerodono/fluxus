use std::sync::Arc;

use owo_colors::OwoColorize;
use tokio::{
    net::TcpListener,
    sync::mpsc,
};

use crate::{
    config::Config,
    data::commands::http::GlobalHttpCommand,
};

pub async fn run_http_listener(
    config: Arc<Config>,
    rx: mpsc::UnboundedReceiver<GlobalHttpCommand>,
) -> eyre::Result<()> {
    let Some(ref http) = config.http else {
        return Ok(());
    };
    let bold_http = "`HTTP`".bold();

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

    tracing::info!("Started {bold_http} server on {}", bound.bold());

    loop {
        let (stream, address) = listener.accept().await?;
        tracing::info!(
            "{} connected to the {bold_http} server",
            address.bold()
        );
    }
}
