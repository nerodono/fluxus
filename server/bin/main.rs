use std::{
    num::NonZeroUsize,
    path::{
        Path,
        PathBuf,
    },
    process,
};

use eyre::Context;
use fluxus::config::{
    Config,
    Error as ConfigError,
    LogLevel,
    LoggingConfig,
};
use owo_colors::OwoColorize;
use tokio::runtime::Builder;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(serde::Deserialize)]
struct EnvParams {
    workers: Option<NonZeroUsize>,
    config_path: Option<PathBuf>,
}

fn main() -> eyre::Result<()> {
    let env_params: EnvParams = envy::prefixed("FLUXUS_")
        .from_env()
        .wrap_err("Failed to parse environment variables")?;
    let workers = env_params
        .workers
        .map_or_else(num_cpus::get, NonZeroUsize::get);
    let config = match env_params.config_path {
        Some(ref path) => Config::try_load(path)
            .wrap_err("Failed to load configuration file")?,

        None => match Config::try_load_paths(&config_paths()) {
            Ok(c) => c,
            Err(e) => print_config_error(e),
        },
    };

    setup_tracing(&config.logging)?;

    let rt = match workers {
        0 | 1 => Builder::new_current_thread(),
        n => {
            let mut b = Builder::new_multi_thread();
            b.worker_threads(n);
            b
        }
    }
    .enable_io()
    .enable_time()
    .build()
    .wrap_err("Failed to create tokio runtime")?;

    rt.block_on(async_entrypoint::entrypoint(config.into()))
}

fn setup_tracing(config: &LoggingConfig) -> eyre::Result<()> {
    if matches!(config.level, LogLevel::Disable) {
        eprintln!("{}", "Logging was disabled".red().bold());
        return Ok(());
    }

    let subscriber = FmtSubscriber::builder()
        .compact()
        .without_time()
        .with_max_level(match config.level {
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Disable => unreachable!(),
            LogLevel::Error => Level::ERROR,
            LogLevel::Info => Level::INFO,
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .wrap_err("Failed to setup tracing")?;

    Ok(())
}

fn config_paths() -> [&'static Path; 4] {
    [
        Path::new("fluxus.toml"),
        Path::new("config.toml"),
        Path::new("assets/config.toml"),
        Path::new("/etc/fluxus.toml"),
    ]
}

fn print_config_error(errors: Vec<ConfigError>) -> ! {
    eprintln!("Failed to load config file, tried paths:");

    let paths = config_paths();
    for (idx, error) in errors.into_iter().enumerate() {
        let path = paths[idx];

        eprintln!("  - {}: {error}", path.display().bold());
    }

    eprintln!("Shutting down...");
    process::exit(1)
}

mod async_entrypoint;
