use std::sync::Arc;

use owo_colors::OwoColorize;
use tokio::net::TcpListener;

use crate::{
    config::Config,
    protocols::galaxy::handler::handle_connection,
    utils::feature_gate::FeatureGate,
};

const NAME: &str = "`Galaxy`";

pub async fn run(config: Arc<Config>, gate: FeatureGate) -> eyre::Result<()> {
    let listener = TcpListener::bind(&config.server.listen).await?;
    let bound_to = listener.local_addr()?; // for cases where port is 0

    let stylized_proto = NAME.bold();
    let stylized_proto = stylized_proto.green();

    tracing::info!("{stylized_proto} listening on {}", bound_to.bold());

    loop {
        let (stream, address) = listener.accept().await?;
        tracing::info!(
            "{} connected to the {stylized_proto} server",
            address.bold()
        );

        let config = Arc::clone(&config);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, address, config).await {
                tracing::info!(
                    "{} disconnected from the {} server: {}",
                    "`Galaxy`".bold().green(),
                    address.bold(),
                    e.bold()
                );
            } else {
                tracing::info!(
                    "{} disconnected from the {} server",
                    address.bold(),
                    "`Galaxy`".bold().green()
                );
            }
        });
    }
}
