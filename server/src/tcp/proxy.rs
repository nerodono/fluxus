use std::{
    net::SocketAddr,
    sync::Arc,
};

use owo_colors::OwoColorize;
use tokio::{
    io::{
        AsyncReadExt,
        AsyncWriteExt,
    },
    net::{
        TcpListener,
        TcpStream,
    },
    sync::mpsc::{
        unbounded_channel,
        UnboundedReceiver,
        UnboundedSender,
    },
};

use crate::logic::{
    command::{
        MasterCommand,
        SlaveCommand,
    },
    shutdown_token::ShutdownListener,
    tcp_server::TcpIdPool,
};

async fn listen_tcp_proxy_client(
    self_id: u16,
    mut stream: TcpStream,
    recv_buffer: usize,

    master_chan: UnboundedSender<MasterCommand>,
    mut self_chan: UnboundedReceiver<SlaveCommand>,
) {
    let mut gracefully = true;
    let mut buffer = vec![0; recv_buffer];
    loop {
        tokio::select! {
            len = stream.read(&mut buffer) => {
                let Ok(len @ 1..) = len else {
                    break;
                };
                let Ok(_) = master_chan.send(MasterCommand::Forward { id: self_id, buffer: Vec::from(&buffer[..len]) }) else {
                    break;
                };
            }

            command = self_chan.recv() => {
                let Some(command) = command else {
                    break;
                };

                match command {
                    SlaveCommand::Disconnect => {
                        gracefully = false;
                        break;
                    }
                    SlaveCommand::Forward { buffer } => {
                        let Ok(_) = stream.write_all(&buffer).await else {
                            break;
                        };
                    }
                }
            }
        }
    }

    if !gracefully {
        let _ =
            master_chan.send(MasterCommand::Disconnected { id: self_id });
    }
}

pub async fn listen_tcp_proxy(
    creator: SocketAddr,
    pool: Arc<TcpIdPool>,
    listener: TcpListener,
    mut token: ShutdownListener,
    chan: UnboundedSender<MasterCommand>,
) {
    let gracefully = true;

    loop {
        let stream: TcpStream;
        let address: SocketAddr;

        tokio::select! {
            biased;
            _ = token.wait_for_cancelation() => {
                break;
            }

            result = listener.accept() => {
                let Ok((i_stream, i_addr)) = result else {
                    break
                };
                stream = i_stream;
                address = i_addr;
            }
        }

        let id = if let Some(id) = pool.lock().await.request() {
            id
        } else {
            tracing::error!(
                "{} Connect would overflow next ID of the {}'s server, \
                 disconnected",
                address.bold(),
                creator.bold()
            );
            continue;
        };

        let (client_tx, client_rx) = unbounded_channel();
        if chan
            .send(MasterCommand::Connected {
                id,
                channel: client_tx,
            })
            .is_err()
        {
            // there is no need in setting gracefully to false since
            // master channel is closed
            break;
        }

        let pool = Arc::clone(&pool);
        let master_chan = chan.clone();
        tokio::spawn(async move {
            listen_tcp_proxy_client(
                id,
                stream,
                4096,
                master_chan,
                client_rx,
            )
            .await;
            tracing::error!(
                "{} Disconnected from the {}'s server",
                address.bold(),
                creator.bold()
            );
            pool.lock().await.return_id(id);
        });
    }

    if !gracefully {
        chan.send(MasterCommand::Stopped)
            .unwrap_or_default();
    }
}
