use std::sync::Arc;

use galaxy_network::{
    error::ReadError,
    reader::GalaxyReader,
    writer::GalaxyWriter,
};
use owo_colors::OwoColorize;
use tokio::net::TcpListener;

use crate::{
    config::Config,
    data::id_pool::create_id_pool,
    protocols::galaxy::handler::handle_connection,
    utils::{
        self,
        feature_gate::FeatureGate,
    },
};

pub async fn run_galaxy_listener(
    config: Arc<Config>,
    feature_gate: FeatureGate,
) -> eyre::Result<()> {
    let bold_galaxy = "`Galaxy`".bold();

    let listen_addr = config.server.listen;
    let listener = TcpListener::bind(listen_addr)
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to start {bold_galaxy} protocol listener on the {}: \
                 {e}",
                listen_addr.bold()
            );
            e
        })?;
    let listen_addr = listener.local_addr()?;

    tracing::info!(
        "{bold_galaxy} protocol server started on {}",
        listen_addr.bold()
    );

    loop {
        let (mut stream, address) = listener.accept().await?;
        let (compressor, decompressor) =
            utils::compression::create_compressor_decompressor(
                &config.compression,
            );
        tracing::info!(
            "{} connected to the {bold_galaxy} server",
            address.bold()
        );

        let config = Arc::clone(&config);
        let gate = feature_gate.clone();

        tokio::spawn(async move {
            let (r_side, w_side) = stream.split();
            let reader = GalaxyReader::new(r_side, decompressor);
            let writer = GalaxyWriter::new(w_side, compressor);

            let result = handle_connection(
                reader,
                writer,
                config,
                address,
                create_id_pool,
                gate,
            )
            .await;
            if matches!(result, Err(ReadError::UnknownPacket)) {
                tracing::error!(
                    "{} disconnected due to incorrect packet type retrieval",
                    address.bold()
                );
            } else {
                tracing::info!(
                    "{} disconnected from the {} server",
                    address.bold(),
                    "`Galaxy`".bold()
                );
            }
        });
    }
}
