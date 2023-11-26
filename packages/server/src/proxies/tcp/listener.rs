use std::{
    net::SocketAddr,
    sync::Arc,
};

use owo_colors::OwoColorize;
use tokio::{
    net::TcpListener,
    sync::{
        mpsc,
        Notify,
    },
};

use super::connection_handler::run_connection_handler;
use crate::protocols::tcp_flux::events::{
    flow::{
        FlowEvent,
        FlowHandshake,
    },
    master::MasterEvent,
};

// TODO: make buffer and channel size configurable
const CHAN_SIZE: usize = 100;
const BUFFER_SIZE: usize = 4096;

pub async fn run_tcp_listener(
    shutdown_token: Arc<Notify>,
    bound_on: SocketAddr,
    listener: TcpListener,
    master_push: mpsc::UnboundedSender<MasterEvent>,
) {
    loop {
        let accept_result = tokio::select! {
            biased;
            _ = shutdown_token.notified() => {
                break;
            }

            accept = listener.accept() => {
                accept
            }
        };
        let (stream, address) = match accept_result {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("{} accept error: {e}", bound_on.bold());
                break;
            }
        };

        tracing::info!("{} connected to the {}", address.bold(), bound_on.bold());

        let (flow_tx, flow_rx) = mpsc::channel(CHAN_SIZE);
        let (master_tx, master_rx) = mpsc::channel(CHAN_SIZE);

        let notifier = Arc::new(Notify::new());
        let handshake = FlowHandshake {
            flow_tx,
            master_rx,
            notifier: Arc::clone(&notifier),
        };

        tokio::spawn(async move {
            _ = run_connection_handler(
                notifier,
                BUFFER_SIZE,
                stream,
                &master_tx,
                flow_rx,
            )
            .await;
            _ = master_tx.send(FlowEvent::Closed).await;
        });
        if master_push
            .send(MasterEvent::Connected { handshake })
            .is_err()
        {
            break;
        }
    }
    _ = master_push.send(MasterEvent::ShutdownServer);
}
