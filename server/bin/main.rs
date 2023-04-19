use std::{
    fmt::Display,
    num::NonZeroUsize,
    path::{
        Path,
        PathBuf,
    },
    process,
    sync::Arc,
};

use eyre::Context;
use neo::config::{
    Config,
    LogLevel,
};
use owo_colors::OwoColorize;
use tokio::{
    runtime,
    task::JoinHandle,
};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[rustfmt::skip]
#[cfg(feature = "galaxy")]
use neo::protocols::galaxy;
#[cfg(feature = "http")]
use neo::protocols::http;

#[rustfmt::skip]
#[cfg(feature = "http")]
use tokio::sync::mpsc;
#[cfg(feature = "http")]
use neo::data::commands::http::GlobalHttpCommand;

#[derive(Debug, serde::Deserialize)]
struct EnvParams {
    /// Custom path for config loading
    config_path: Option<PathBuf>,

    /// Number of neogrok workers. Defaults to number of
    /// logical CPUs
    workers: Option<NonZeroUsize>,
}

#[allow(clippy::vec_init_then_push)]
async fn async_main(config: Config) -> eyre::Result<()> {
    let config = Arc::new(config);
    let mut handles = Vec::new();

    #[cfg(feature = "http")]
    let http_tx = {
        let (tx, rx) = mpsc::unbounded_channel();
        handles.push(start_http(Arc::clone(&config), rx));
        tx
    };

    #[cfg(feature = "galaxy")]
    handles.push(start_galaxy(
        config,
        #[cfg(feature = "http")]
        http_tx,
    ));

    for handle in handles {
        _ = handle.await;
    }

    Ok(())
}

fn main() -> eyre::Result<()> {
    let params: EnvParams = match envy::prefixed("NEOGROK_").from_env() {
        Ok(p) => p,
        Err(e) => die("Failed to parse environment", e),
    };
    let config = load_config(&params.config_path);
    let runtime = create_runtime(params.workers)?;

    setup_tracing(&config)?;

    runtime.block_on(async_main(config))
}

// Starters

#[cfg(feature = "http")]
fn start_http(
    config: Arc<Config>,
    rx: mpsc::UnboundedReceiver<GlobalHttpCommand>,
) -> JoinHandle<eyre::Result<()>> {
    tokio::spawn(http::listener::run_http_listener(config, rx))
}

#[cfg(feature = "galaxy")]
fn start_galaxy(
    config: Arc<Config>,
    #[cfg(feature = "http")] http_chan: mpsc::UnboundedSender<
        GlobalHttpCommand,
    >,
) -> JoinHandle<eyre::Result<()>> {
    tokio::spawn(galaxy::listener::run_galaxy_listener(
        config,
        #[cfg(feature = "http")]
        http_chan,
    ))
}

//

fn die(prelude: &str, e: impl Display) -> ! {
    eprintln!("{} {}: {e}", "!!".red().bold(), prelude.bold());
    process::exit(1)
}

fn setup_tracing(config: &Config) -> eyre::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(match config.logging.level {
            LogLevel::Info => Level::INFO,
            LogLevel::Disable => {
                // FIXME: Implement disable
                eprintln!(
                    "{} {}",
                    "!!".red().bold(),
                    "Currently `disable` level is not supported, falling \
                     back to `info`"
                        .bold()
                );

                Level::INFO
            }
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Error => Level::ERROR,
        })
        .without_time()
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .wrap_err("setting default subscriber failed")
}

fn create_runtime(
    workers: Option<NonZeroUsize>,
) -> eyre::Result<runtime::Runtime> {
    match workers
        .map(NonZeroUsize::get)
        .unwrap_or(num_cpus::get())
    {
        0 | 1 => runtime::Builder::new_current_thread(),
        n => {
            let mut b = runtime::Builder::new_multi_thread();
            b.worker_threads(n);
            b
        }
    }
    .enable_io()
    .build()
    .wrap_err("Failed to create tokio runtime")
}

fn load_config(path: &Option<PathBuf>) -> Config {
    let default_paths: &[&Path] = &[
        Path::new("neogrok.toml"),
        Path::new("assets/config.toml"),
        Path::new("config.toml"),
        Path::new("/etc/neogrok.toml"),
        Path::new("/etc/neogrok/config.toml"),
    ];

    match path {
        Some(ref buf) => match Config::try_load(buf) {
            Ok(c) => c,
            Err(e) => die("Failed to parse supplied config file", e),
        },

        None => match Config::try_load_paths(default_paths) {
            Ok(c) => c,
            Err(errors) => {
                eprintln!(
                    "{} {}",
                    "!!".red().bold(),
                    "Failed to locate and parse config file:".bold()
                );
                for (&path, error) in
                    default_paths.iter().zip(errors.into_iter())
                {
                    eprintln!("  - {}: {error}", path.display().green());
                }

                process::exit(1)
            }
        },
    }
}
