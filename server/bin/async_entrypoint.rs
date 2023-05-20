use std::sync::Arc;

use fluxus::{
    config::Config,
    data::services::Services,
    server::listener::run_flux_listener,
};
use owo_colors::OwoColorize;

pub async fn async_main(config: Arc<Config>) {
    let mut services = Services::default();

    services.add_service("flux", run_flux_listener(config));

    while let Some((name, join_result)) = services.next_shutdown().await {
        let stylized = name.bold();
        let stylized = stylized.green();

        let result = match join_result {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("failed to join {stylized}: {e}");
                continue;
            }
        };
        let error = result.err().map_or_else(
            || String::from("no errors reported"),
            |e| format!("{e}"),
        );

        tracing::info!("{stylized} finished: {error}");
    }

    tracing::info!("server will shut down now");
}
