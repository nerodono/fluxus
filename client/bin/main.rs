use std::num::NonZeroUsize;

use fluxus::cli_args::CliArgs;
use tokio::runtime::Builder;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

fn with_workers_no(no: usize) -> Builder {
    let mut b = Builder::new_multi_thread();
    b.worker_threads(no);
    b
}

fn main() -> eyre::Result<()> {
    let args = CliArgs::parse();
    setup_tracing();

    let runtime = match args.workers.map(NonZeroUsize::get) {
        Some(1) => Builder::new_current_thread(),

        None => with_workers_no(num_cpus::get()),
        Some(n) => with_workers_no(n),
    }
    .enable_io()
    .build()?;

    runtime.block_on(async_entrypoint::async_main(args))
}

fn setup_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .compact()
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set default subscriber");
}

mod async_entrypoint;
