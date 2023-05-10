use std::{
    io,
    net::SocketAddr,
    num::NonZeroUsize,
};

use idpool::interface::IdPool;
use owo_colors::OwoColorize;
use tokio::{
    net::TcpListener,
    sync::mpsc,
};

use crate::{
    data::{
        commands::{
            master::TcpPermit,
            tcp::TcpMasterCommand,
        },
        proxy::Pool,
    },
    slaves::tcp::handler::handle_connection,
    utils::shutdown_token::ShutdownToken,
};

pub async fn listen(
    creator: SocketAddr,
    listener: TcpListener,
    bound_on: SocketAddr,
    permit: TcpPermit,
    pool: Pool,
    read_buffer: usize,
    channel_capacity: NonZeroUsize,
    mut token: ShutdownToken,
) -> io::Result<()> {
    tracing::info!(
        "Started {}'s TCP server on {}",
        creator.bold(),
        bound_on.bold()
    );
    loop {
        let stream;
        let address;

        tokio::select! {
            biased;

            _ = token.wait_for_shutdown() => {
                tracing::info!("{}'s TCP server received shutdown signal", creator.bold());
                break;
            }

            result = listener.accept() => {
                (stream, address) = result?;
            }
        }

        let Some(id) = pool.lock().await.request() else {
            continue;
        };
        let (tx, rx) = mpsc::channel(channel_capacity.get());

        if permit
            .send(TcpMasterCommand::Connected { id, chan: tx })
            .await
            .is_err()
        {
            break;
        }

        tracing::info!(
            "{} connected to the {}'s TCP server (ID = {id})",
            address.bold(),
            creator.bold()
        );

        let pool = pool.clone();
        let permit = permit.clone();
        tokio::spawn(async move {
            _ = handle_connection(id, rx, read_buffer, stream, permit).await;
            pool.lock().await.return_id(id);
        });
    }

    _ = permit.send(TcpMasterCommand::Stopped).await;

    Ok(())
}
