use std::net::SocketAddr;

use owo_colors::OwoColorize;
use tokio::{
    io::{
        AsyncReadExt,
        AsyncWriteExt,
    },
    net::TcpStream,
    sync::mpsc::{
        UnboundedReceiver,
        UnboundedSender,
    },
};

use super::command::{
    MasterCommand,
    SlaveCommand,
};

pub async fn run_slave(
    self_id: u16,
    buffer_size: usize,
    connect_to: SocketAddr,
    master: UnboundedSender<MasterCommand>,
    mut chan: UnboundedReceiver<SlaveCommand>,
) {
    let mut stream = match TcpStream::connect(connect_to).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(
                "Failed to connect to the {}: {e}",
                connect_to.bold()
            );
            let _ = master.send(MasterCommand::Disconnect { id: self_id });
            return;
        }
    };

    let mut gracefully = false;
    let mut buffer = vec![0; buffer_size];

    loop {
        tokio::select! {
            read = stream.read(&mut buffer) => {
                let Ok(read @ 1..) = read else {
                    break;
                };
                if master.send(
                    MasterCommand::Forward {
                        id: self_id,
                        buffer: Vec::from(&buffer[..read])
                    }
                ).is_err() {
                    break;
                }
            }

            command = chan.recv() => {
                let Some(command) = command else {
                    break;
                };

                match command {
                    SlaveCommand::Disconnect => {
                        gracefully = true;
                        break;
                    }

                    SlaveCommand::Forward { buffer: master_buf } => {
                        if let Err(e) = stream.write_all(&master_buf).await {
                            tracing::error!(
                                "Failed to forward data to the {}: {e}", connect_to.bold());
                            break;
                        }
                    }
                }
            }
        }
    }

    if !gracefully {
        let _ = master.send(MasterCommand::Disconnect { id: self_id });
    }
}
