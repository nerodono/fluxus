use futures::{
    stream::FuturesUnordered,
    Future,
    StreamExt,
};
use tokio::task::JoinError;

use crate::utils::named_join_handle::NamedJoinHandle;

#[derive(Default)]
pub struct Services {
    futures: FuturesUnordered<NamedJoinHandle<eyre::Result<()>>>,
}

impl Services {
    pub async fn next_shutdown(
        &mut self,
    ) -> Option<(&'static str, Result<eyre::Result<()>, JoinError>)> {
        self.futures.next().await
    }

    pub fn add_service<F>(&self, name: &'static str, future: F)
    where
        F: Send + Future<Output = eyre::Result<()>> + 'static,
    {
        self.futures
            .push(NamedJoinHandle::new(name, tokio::spawn(future)));
    }
}
