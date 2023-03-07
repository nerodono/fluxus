use mimalloc::MiMalloc;
use neogrok::{
    config::{
        Config,
        ParserFunctionality,
    },
    server,
};
use tokio::{
    net::TcpListener,
    runtime::Builder,
};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

async fn async_main(config: Config) -> anyhow::Result<()> {
    let listener = TcpListener::bind(&config.server.listen).await?;
    let address = listener.local_addr()?;

    tracing::info!("Started TCP server at {address}");

    server::spawner::run_tcp_spawner(listener, config).await?;
    Ok(())
}

fn setup_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set default subscriber");
}

fn main() -> anyhow::Result<()> {
    setup_tracing();
    let config = Config::try_load("assets/config/config.toml")?;
    let runtime = match config.server.worker_threads.get() {
        0..=1 => Builder::new_current_thread(),
        otherwise => {
            let mut b = Builder::new_multi_thread();
            b.worker_threads(otherwise);
            b
        }
    }
    .enable_io()
    .thread_name("Neogrok worker")
    .build()?;

    runtime.block_on(async_main(config))
}
