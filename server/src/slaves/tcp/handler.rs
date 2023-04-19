use std::io;

use tokio::{
    io::{
        AsyncReadExt,
        AsyncWriteExt,
    },
    net::TcpStream,
    sync::mpsc,
};

use crate::data::commands::{
    base::TcpPermit,
    tcp::{
        TcpMasterCommand,
        TcpSlaveCommand,
    },
};

pub async fn run_tcp_slave_handler(
    id: u16,
    mut chan: mpsc::UnboundedReceiver<TcpSlaveCommand>,
    permit: TcpPermit,
    mut stream: TcpStream,
    allocate: usize,
) -> io::Result<()> {
    let mut gracefully = false;
    let mut buffer = vec![0; allocate];

    loop {
        tokio::select! {
            command = chan.recv() => {
                let Some(command) = command else {
                    break;
                };
                match command {
                    TcpSlaveCommand::Disconnect => {
                        gracefully = true;
                        break;
                    }

                    TcpSlaveCommand::Forward { buffer } => {
                        stream.write_all(&buffer).await?;
                    }
                }
            }

            rd = stream.read(&mut buffer) => {
                let Ok(read @ 1..) = rd else {
                    break;
                };

                let newly_allocated = Vec::from(&buffer[..read]);
                if permit.send(TcpMasterCommand::Forward { id, buffer: newly_allocated }).is_err() {
                    gracefully = true;
                    break;
                }
            }
        }
    }

    if !gracefully {
        _ = permit.send(TcpMasterCommand::Disconnect { id });
    }

    Ok(())
}
