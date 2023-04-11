use std::{
    io,
    sync::Arc,
};

use owo_colors::OwoColorize;
use tokio::net::TcpListener;

use crate::config::Config;

pub async fn run_listener(config: Arc<Config>) -> io::Result<()> {
    let listener = TcpListener::bind(&config.server.listen).await?;
    let listener_address = listener.local_addr()?;
    tracing::info!(
        "Started {} protocol listener at {}",
        "Galaxy".bold(),
        listener_address.bold()
    );

    loop {
        let (stream, address) = listener.accept().await?;
        if let Err(e) = stream.set_nodelay(true) {
            tracing::error!(
                "Failed to disable {} for the {} (latency can be \
                 increased): {e}",
                "Nagle algorithm".bold(),
                address.bold()
            );
        }
    }
}
