use std::net::SocketAddr;

use idpool::interface::IdPool;
use owo_colors::OwoColorize;
use tokio::{
    net::TcpListener,
    sync::mpsc,
};

use crate::{
    data::{
        commands::{
            base::TcpPermit,
            tcp::TcpMasterCommand,
        },
        id_pool::{
            clone_id_pool,
            IdPoolImpl,
        },
    },
    slaves::tcp::handler::run_tcp_slave_handler,
    utils::shutdown_token::ShutdownToken,
};

pub async fn tcp_slave_listen(
    permit: TcpPermit,
    id_pool: IdPoolImpl,
    mut token: ShutdownToken,
    listener: TcpListener,
    creator: SocketAddr,
    allocate_per_client: usize,
) {
    loop {
        let stream;
        let address;
        tokio::select! {
            biased;

            _ = token.wait_for_shutdown() => {
                break;
            }

            result = listener.accept() => {
                (stream, address) = match result {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::info!("{} failed to accept TCP connection: {e}", creator.bold());
                        break;
                    }
                };
            }
        }

        let Some(id) = id_pool.lock().await.request() else {
            tracing::error!(
                "{} would overflow ID value of the {}'s TCP server: closing \
                 connection",
                address.bold(),
                creator.bold()
            );
            continue;
        };

        tracing::info!(
            "{} connected to the {}'s TCP server (ID = {id})",
            address.bold(),
            creator.bold()
        );

        let (tx, rx) = mpsc::unbounded_channel();

        if permit
            .send(TcpMasterCommand::Connect { id, chan: tx })
            .is_err()
        {
            break;
        }

        let permit = permit.clone();
        let pool = clone_id_pool(&id_pool);
        tokio::spawn(async move {
            _ = run_tcp_slave_handler(
                id,
                rx,
                permit,
                stream,
                allocate_per_client,
            )
            .await;
            pool.lock().await.return_id(id);

            tracing::info!(
                "{} disconnected from the {}'s server (ID = {id})",
                address.bold(),
                creator.bold()
            );
        });
    }

    _ = permit.send(TcpMasterCommand::Stopped);
}
