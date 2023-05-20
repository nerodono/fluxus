use std::{
    net::SocketAddr,
    sync::Arc,
};

use flux_compression::polymorphic::{
    PolyCctx,
    PolyDctx,
};
use owo_colors::OwoColorize;
use tokio::{
    net::TcpListener,
    sync::mpsc,
};

use crate::{
    communication::control::ControlCommand,
    config::{
        CompressionScope,
        Config,
    },
    data::server::{
        QueuedConnection,
        Server,
    },
    server::slave::handler::run_tcp_slave_handler,
    utils::shutdown_token::ShutdownReader,
};

pub async fn run_tcp_slave_listener<Fd, F>(
    cctx_factory: F,
    dctx_factory: Fd,

    server: Arc<Server>,
    config: Arc<Config>,
    creator: SocketAddr,
    mut shutdown_reader: ShutdownReader,
    listener: TcpListener,

    control_tx: mpsc::Sender<ControlCommand>,
) where
    F: Fn(&CompressionScope) -> PolyCctx,
    Fd: Fn(&CompressionScope) -> PolyDctx,
{
    loop {
        let stream;
        let address;

        tokio::select! {
            biased;

            _ = shutdown_reader.read() => {
                break;
            }

            result = listener.accept() => {
                (stream, address) = match result {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!("{} failed to accept connection: {e}", creator.bold());
                        break;
                    }
                };
            }
        }

        let (slave_tx, slave_rx) =
            mpsc::channel(config.buffering.channels.slave.get());
        let (flow_tx, flow_rx) =
            mpsc::channel(config.buffering.channels.flow.get());

        let (cctx, dctx) = if let Some(ref scope) = config.compression.tcp {
            (Some(cctx_factory(scope)), Some(dctx_factory(scope)))
        } else {
            (None, None)
        };

        let connection = QueuedConnection {
            slave_tx,
            flow_rx,
            dctx,
        };
        if let Err(e) = server.enqueue_single(connection).await {
            tracing::error!(
                "{} failed to accept connection: {e}",
                address.bold()
            );
            continue;
        }

        let scope = config.compression.tcp.clone();
        tokio::spawn(run_tcp_slave_handler(
            config.buffering.read.per_slave.get(),
            creator,
            address,
            stream,
            cctx.zip(scope),
            flow_tx,
            slave_rx,
        ));

        if control_tx
            .send(ControlCommand::Connected)
            .await
            .is_err()
        {
            break;
        }
    }

    _ = control_tx.send(ControlCommand::Shutdown).await;
}
