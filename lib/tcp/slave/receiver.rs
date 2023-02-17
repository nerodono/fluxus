use std::net::SocketAddr;

use tokio::{
    io::{
        AsyncReadExt,
        AsyncWriteExt,
    },
    net::TcpStream,
};

use crate::{
    config::base::Config,
    tcp::handlers::message_types::{
        MasterMessage,
        SlaveMessage,
    },
};

/// Slave's client TCP Listener.
pub async fn listen_for_tcp_client(
    mut stream: TcpStream,
    creator: SocketAddr,
    address: SocketAddr,

    rx: flume::Receiver<SlaveMessage>,
    master: flume::Sender<MasterMessage>,

    self_id: u16,
) {
    let per_client_buffer_size =
        Config::instance().server.bufferization.per_client;
    let mut buffer: Vec<u8> = vec![0; per_client_buffer_size];
    let mut forcibly_disconnected = false;

    loop {
        tokio::select! {
            message = rx.recv_async() => {
                let Ok(message) = message else {
                    break;
                };

                match message {
                    SlaveMessage::Disconnect => {
                        tracing::info!(
                            "{address} was forcibly disconnected from the {creator}"
                        );
                        forcibly_disconnected = true;
                        break;
                    }

                    SlaveMessage::Forward { data } => {
                        let Ok(_) = stream.write_all(&data).await else {
                            break;
                        };
                    }
                }
            }

            read = stream.read(&mut buffer) => {
                let Ok(read @ 1..) = read else {
                    break;
                };
                let Ok(_) = master.send_async(MasterMessage::Forward {
                    id: self_id,
                    data: Vec::from(&buffer[..read])
                }).await else {
                    break;
                };
            }
        }
    }

    if !forcibly_disconnected {
        tracing::info!(
            %creator,
            "{address} is disconnected"
        );
        master
            .send_async(MasterMessage::Disconnected { id: self_id })
            .await
            .unwrap_or(());
    }
}
