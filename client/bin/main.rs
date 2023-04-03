use std::{
    fmt::Display,
    num::NonZeroUsize,
    process,
};

use args::CliArgs;
use clap::Parser;
use owo_colors::OwoColorize;
use tokio::runtime::Builder;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, serde::Deserialize)]
struct EnvParams {
    workers: Option<NonZeroUsize>,
}

async fn async_main(args: CliArgs) -> eyre::Result<()> {
    todo!()
}

fn main() -> eyre::Result<()> {
    let params: EnvParams = match envy::prefixed("NEO_").from_env() {
        Ok(p) => p,
        Err(e) => die("Failed to parse env params", e),
    };
    let args = CliArgs::parse();

    setup_tracing();
    let res_rt = match params
        .workers
        .or(NonZeroUsize::new(num_cpus::get()))
        .map(NonZeroUsize::get)
    {
        Some(0 | 1) | None => Builder::new_current_thread(),

        Some(n) => {
            let mut b = Builder::new_multi_thread();
            b.worker_threads(n);
            b
        }
    }
    .enable_io()
    .build();

    match res_rt {
        Ok(rt) => rt.block_on(async_main(args)),
        Err(e) => die("Failed to create tokio runtime", e),
    }
}

fn die(display: impl Display, error: impl Display) -> ! {
    eprintln!("{} {}: {}", "!!".red().bold(), display.bold(), error);
    process::exit(1)
}

fn setup_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set default subscriber");
}

mod args;
