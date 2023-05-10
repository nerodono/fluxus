use tokio::{
    io::{
        AsyncReadExt,
        AsyncWriteExt,
    },
    net::TcpStream,
    sync::mpsc,
};

use crate::data::commands::{
    master::TcpPermit,
    tcp::{
        TcpMasterCommand,
        TcpSlaveCommand,
    },
};

pub async fn handle_connection(
    id: u16,
    mut rx: mpsc::Receiver<TcpSlaveCommand>,
    read_buffer: usize,
    mut stream: TcpStream,
    permit: TcpPermit,
) {
    let mut buffer = vec![0; read_buffer];
    let mut gracefully = false;

    loop {
        tokio::select! {
            command = rx.recv() => {
                let Some(command) = command else {
                    break;
                };

                match command {
                    TcpSlaveCommand::Forward { buffer: fwd_buf } => {
                        if stream.write_all(&fwd_buf).await.is_err() {
                            break;
                        }
                    }
                    TcpSlaveCommand::Disconnect => {
                        gracefully = true;
                        break;
                    }
                }
            }
            read = stream.read(&mut buffer) => {
                let Ok(read @ 1..) = read else {
                    break;
                };

                let buffer = Vec::from(&buffer[..read]);
                if permit.send(TcpMasterCommand::Forward { id, buffer }).await.is_err() {
                    break;
                }
            }
        }
    }

    if !gracefully {
        _ = permit
            .send(TcpMasterCommand::Disconnected { id })
            .await;
    }
}
