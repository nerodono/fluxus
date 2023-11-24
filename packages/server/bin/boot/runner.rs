use std::future::Future;

use color_eyre::eyre;
use tokio::task::JoinHandle;

/// Runs future and displays errors when they occur
pub fn run_fut<F>(proto: &'static str, fut: F) -> JoinHandle<Result<(), eyre::Error>>
where
    F: Future<Output = eyre::Result<()>> + Send + 'static,
{
    tokio::spawn(async move {
        let result = fut.await;
        if let Err(ref report) = result {
            tracing::error!("{proto} exited with an error:\n{report}");
        }

        result
    })
}
