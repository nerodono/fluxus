use std::num::NonZeroUsize;

use color_eyre::eyre;
use tokio::runtime::{
    Builder,
    Runtime,
};

pub fn create_runtime(
    worker_threads: Option<NonZeroUsize>,
) -> eyre::Result<Runtime> {
    let threads = worker_threads.map_or_else(num_cpus::get, NonZeroUsize::get);
    let mut builder = match threads {
        0 | 1 => {
            tracing::debug!("Picking single-threaded runtime");
            Builder::new_current_thread()
        }

        n => {
            tracing::debug!("Picking multi-threaded runtime ({threads} threads)");
            let mut b = Builder::new_multi_thread();
            b.worker_threads(n);
            b
        }
    };

    builder
        .thread_name("fluxus worker")
        .enable_all()
        .build()
        .map_err(From::from)
}
