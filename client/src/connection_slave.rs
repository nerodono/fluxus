use tokio::{
    io::{
        AsyncReadExt,
        AsyncWriteExt,
    },
    net::TcpStream,
    sync::mpsc,
};

use crate::command::{
    Command,
    IdentifiedCommand,
};

pub async fn run_slave(
    id: u16,
    mut stream: TcpStream,
    master: mpsc::UnboundedSender<IdentifiedCommand>,
    mut slave: mpsc::UnboundedReceiver<Command>,
    allocate: usize,
) {
    let mut gracefully = false;
    let mut buffer = vec![0; allocate];
    loop {
        let read_bytes;
        tokio::select! {
            read = stream.read(&mut buffer) => {
                read_bytes = if let Ok(r @ 1..) = read {
                    r
                } else {
                    break;
                };
            }

            command = slave.recv() => {
                let Some(command) = command else {
                    break;
                };

                match command {
                    Command::Disconnect => {
                        gracefully = true;
                        break;
                    }

                    Command::Forward { buffer } => {
                        if stream.write_all(&buffer).await.is_err() {
                            break;
                        }
                    }
                }

                continue;
            }
        }

        if master
            .send(
                Command::Forward {
                    buffer: Vec::from(&buffer[..read_bytes]),
                }
                .identified_by(id),
            )
            .is_err()
        {
            break;
        }
    }

    if !gracefully {
        _ = master.send(Command::Disconnect.identified_by(id));
    }
}
