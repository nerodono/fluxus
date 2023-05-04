use std::{
    pin::Pin,
    sync::Arc,
};

use cfg_if::cfg_if;
use fluxus::{
    config::Config,
    protocols,
    utils::{
        feature_gate::FeatureGate,
        named_join_handle::NamedJoinHandle,
    },
};
use futures::{
    future::poll_fn,
    stream::{
        FuturesUnordered,
        Stream,
    },
};
use owo_colors::OwoColorize;

cfg_if! {
    if #[cfg(feature = "http")] {
        use fluxus::http;
    }
}

pub async fn entrypoint(config: Arc<Config>) -> eyre::Result<()> {
    let mut futures: FuturesUnordered<NamedJoinHandle<eyre::Result<()>>> =
        FuturesUnordered::new();

    cfg_if! {
        if #[cfg(feature = "http")] {
            let (http_feature, future) = http::listener::spawn_listener(Arc::clone(&config));
            futures.push(future);
        }
    }

    let gate = FeatureGate::new(
        #[cfg(feature = "http")]
        http_feature,
    );

    cfg_if! {
        if #[cfg(feature = "galaxy")] {
            futures.push(NamedJoinHandle {
                name: "galaxy",
                handle: tokio::spawn(protocols::galaxy::listener::run(Arc::clone(&config), gate))
            });
        }
    }

    drop(config);

    while let Some((name, result)) =
        poll_fn(|cx| Pin::new(&mut futures).poll_next(cx)).await
    {
        let stylized = name.bold();
        let stylized = stylized.green();

        let result = match result {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Failed to join {stylized}: {e}");
                continue;
            }
        };
        match result {
            Ok(()) => {
                tracing::info!("{stylized} stopped without errors");
            }

            Err(e) => {
                tracing::error!("{stylized} stopped with an error: {e}");
            }
        }
    }

    Ok(())
}
