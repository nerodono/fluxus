use std::path::{
    Path,
    PathBuf,
};

use eyre::Context;
use fluxus::{
    config::{
        Config,
        LoggingConfig,
    },
    traits::FromConfig,
};
use owo_colors::OwoColorize;
use tokio::runtime::Builder;
use tracing_subscriber::FmtSubscriber;

#[derive(serde::Deserialize)]
struct Env {
    config: Option<PathBuf>,
}

fn default_config_path() -> &'static Path {
    Path::new("config.toml")
}

fn setup_tracing(config: &LoggingConfig) -> eyre::Result<()> {
    let Some(level) = Option::from_config(&config.level) else {
        eprintln!("{} logging was disabled in the configuration file", "!!".bold().red());
        return Ok(());
    };

    let subscriber = FmtSubscriber::builder()
        .compact()
        .without_time()
        .with_max_level(level)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .wrap_err("failed to set default tracing subscriber")?;

    Ok(())
}

fn main() -> eyre::Result<()> {
    let env: Env = envy::prefixed("FLUXUS_")
        .from_env()
        .wrap_err("failed to parse environment variables")?;
    let config_path = env
        .config
        .unwrap_or_else(|| default_config_path().to_owned());
    let config = Config::load_config(config_path)?;
    let rt = match config.runtime.workers.get() {
        1 | 0 => Builder::new_current_thread(),
        n => {
            let mut b = Builder::new_multi_thread();
            b.worker_threads(n);
            b
        }
    }
    .enable_io()
    .build()
    .wrap_err("failed to create tokio async runtime")?;

    setup_tracing(&config.logging)?;

    rt.block_on(async_entrypoint::async_main(config.into()));
    Ok(())
}

mod async_entrypoint;
