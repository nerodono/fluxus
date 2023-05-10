use std::sync::Arc;

use eyre::Context;
use owo_colors::OwoColorize;
use tokio::{
    net::TcpListener,
    sync::mpsc,
};

use crate::{
    config::Config,
    data::{
        commands::http::{
            HttpMasterCommand,
            HttpServerRequest,
        },
        http::collection::EndpointCollection,
    },
    decl::continue_,
    http::connection::Connection,
    utils::{
        features::http::HttpChannel,
        named_join_handle::NamedJoinHandle,
    },
};

async fn listen(
    mut rx: mpsc::UnboundedReceiver<HttpServerRequest>,
    config: Arc<Config>,
) -> eyre::Result<()> {
    let Some(ref http) = config.http else {
        tracing::warn!("HTTP proxy was disabled in the config");
        return Ok(());
    };

    let listener = TcpListener::bind(http.listen)
        .await
        .wrap_err("Failed to bind HTTP port")?;
    let bound_to = listener.local_addr()?;
    let buffer_size = config.server.buffering.read.get();
    let discovery_method = http.discovery_method;
    let channel_buffer = config.server.buffering.channels;

    let collection = Arc::new(EndpointCollection::new());

    tracing::info!(
        "Started {} server on {}",
        "HTTP".bold().green(),
        bound_to.bold()
    );

    loop {
        let mut stream;
        let address;
        tokio::select! {
            request = rx.recv() => {
                let Some(request) = request else {
                    break;
                };

                match request {
                    HttpServerRequest::Bind { endpoint, permit, pool } => {
                        // TODO: random endpoint selection
                        let endpoint = endpoint.unwrap();
                        let insertion_result = collection.try_insert_endpoint(
                            pool,
                            endpoint,
                            permit.clone()
                        ).await;
                        match insertion_result {
                            Ok(()) => {
                                continue_!(permit.send(
                                    HttpMasterCommand::Bound { on: None }.unidentified()
                                ).await);
                            }

                            Err(_) => {
                                continue_!(permit.send(
                                    HttpMasterCommand::FailedToBind.unidentified()
                                ).await);
                            }
                        }
                    }

                    HttpServerRequest::Unbind { endpoint } => {
                        _ = collection.try_erase_entrypoint(&endpoint).await;
                    }
                }

                continue;
            }

            result = listener.accept() => {
                (stream, address) = result?;
            }
        }
        tracing::info!("{} connected to the HTTP server", address.bold());

        let collection = Arc::clone(&collection);
        tokio::spawn(async move {
            let (reader, writer) = stream.split();
            let mut connection = Connection::new(
                reader,
                writer,
                buffer_size,
                discovery_method,
                collection,
                channel_buffer,
            );
            let run_result = connection.run().await;
            match run_result {
                Ok(()) => {
                    tracing::info!(
                        "{} disconnected from the HTTP server",
                        address.bold()
                    );
                }

                Err(e) => {
                    tracing::error!(
                        "{} disconnected from the HTTP server with an \
                         error: {e}",
                        address.bold()
                    );
                }
            }
        });
    }

    tracing::error!(
        "All senders of the HTTP master requests channel was dropped, \
         shutting down..."
    );

    Ok(())
}

pub fn spawn_listener(
    config: Arc<Config>,
) -> (HttpChannel, NamedJoinHandle<eyre::Result<()>>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let future = tokio::spawn(listen(rx, config));
    (
        HttpChannel::new(tx),
        NamedJoinHandle {
            name: "http",
            handle: future,
        },
    )
}
