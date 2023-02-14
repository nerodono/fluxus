use std::io;

use neogrok::{
    config::base::*,
    tcp,
};
use tokio::{
    net::TcpListener,
    runtime::Builder,
};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

async fn entrypoint(listen: &str) -> io::Result<()> {
    let tcp_listener = TcpListener::bind(listen).await?;
    tcp::listener::run_tcp_listener(tcp_listener).await?;

    Ok(())
}

fn configure_logger() {
    let subscriber = FmtSubscriber::builder()
        .pretty()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global tracing subscriber");
}

fn main() -> io::Result<()> {
    let config = Config::instance();
    configure_logger();

    let rt = match config.server.threads {
        Some(1 | 0) | None => Builder::new_current_thread(),
        Some(n) => {
            let mut b = Builder::new_multi_thread();
            b.worker_threads(n);
            b
        }
    }
    .thread_name("neogrok-worker")
    .enable_io()
    .build()
    .expect("Failed to create tokio runtime");

    rt.block_on(entrypoint(&config.server.listen))
}
