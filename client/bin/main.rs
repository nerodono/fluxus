use std::{
    fmt::Display,
    net::{
        SocketAddr,
        ToSocketAddrs,
    },
    num::NonZeroUsize,
    process,
};

use args::{
    CliArgs,
    CliSub,
};
use clap::Parser;
use neo::tcp;
use owo_colors::OwoColorize;
use tokio::{
    net::TcpStream,
    runtime::Builder,
};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, serde::Deserialize)]
struct EnvParams {
    workers: Option<NonZeroUsize>,
}

fn prefer_ipv(s: impl ToSocketAddrs, prefer_ipv4: bool) -> SocketAddr {
    let mut addrs = match s.to_socket_addrs() {
        Ok(a) => a,
        Err(e) => die("Failed to parse local address", e),
    };
    let mut selected = addrs
        .next()
        .expect("No IP address associated with host");

    // Silly code, but I simply don't care :D
    for addr in addrs {
        match addr {
            SocketAddr::V4(..) => {
                if prefer_ipv4 {
                    selected = addr;
                    break;
                }
            }

            SocketAddr::V6(..) => {
                if !prefer_ipv4 {
                    selected = addr;
                    break;
                }
            }
        }
    }
    selected
}

async fn async_main(args: CliArgs) -> eyre::Result<()> {
    match args.sub {
        CliSub::Tcp { local, port } => {
            tracing::info!("Connecting to the {}...", args.remote.bold());
            let remote = TcpStream::connect(&args.remote)
                .await
                .unwrap_or_else(|e| {
                    die("Failed to connect to the remote", e)
                });
            let local = prefer_ipv(local, !args.prefer_ipv6);

            tcp::init::run_work(
                remote,
                args.password.as_deref(),
                &args.remote,
                local,
                port,
            )
            .await
        }
    }
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
