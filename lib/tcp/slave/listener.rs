use std::net::SocketAddr;

use tokio::{
    net::TcpListener,
    sync::oneshot,
};

use crate::tcp::{
    handlers::message_types::{
        MasterMessage,
        ShutdownToken,
    },
    slave::receiver,
    views::MasterStateView,
};

/// TCP Listener slave
pub async fn run_slave_tcp_listener(
    listener: TcpListener,
    creator: SocketAddr,
    mut token: oneshot::Receiver<ShutdownToken>,
    view: MasterStateView,
) {
    loop {
        tokio::select! {
            biased;

            _ = &mut token => {
                break;
            }

            result = listener.accept() => {
                let (stream, address) = match result {
                    Ok(l) => l,
                    Err(error) => {
                        tracing::error!(
                            %creator,
                            %error,
                            "Failed to accept incoming connection"
                        );
                        break;
                    }
                };

                match view.pool.lock().await.request() {
                    Ok(id) => {
                        let (tx, rx) = flume::unbounded();
                        let Ok(_) = view.master.send_async(
                            MasterMessage::Connected { id, tx }
                        ).await else {
                            break;
                        };

                        tokio::spawn(receiver::listen_for_tcp_client(
                            stream,
                            creator,
                            address,
                            rx,
                            view.master.clone(),
                            id
                        ));
                    }

                    Err(e) => {
                        tracing::error!(
                            error = %e,
                            %creator,
                            "Failed to accept incoming connection to the proxy"
                        );

                        continue;
                    }
                }
            }
        }
    }

    view.master
        .send_async(MasterMessage::Shutdown)
        .await
        .unwrap_or_default();
}
