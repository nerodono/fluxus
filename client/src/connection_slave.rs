use std::sync::Arc;

use owo_colors::OwoColorize;
use tokio::{
    io::{
        AsyncReadExt,
        AsyncWriteExt,
    },
    net::TcpStream,
    sync::{
        mpsc,
        Semaphore,
    },
};

use crate::command::{
    Command,
    IdentifiedCommand,
};

pub async fn run_slave(
    id: u16,
    mut stream: TcpStream,
    master: mpsc::Sender<IdentifiedCommand>,
    mut slave: mpsc::Receiver<Command>,
    allocate: usize,
) {
    let semaphore = Arc::new(Semaphore::const_new(1024));
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

        let vec = Vec::from(&buffer[..read_bytes]);
        let sem_permit = semaphore.clone().acquire_owned().await.unwrap();
        if master
            .send(
                Command::Forward { buffer: vec }
                    .identified_by(sem_permit, id),
            )
            .await
            .is_err()
        {
            break;
        }
    }

    if gracefully {
        tracing::info!(
            "{} disconnected by the client",
            format_args!("ID{id}").bold()
        );
    } else {
        tracing::info!(
            "{} disconnected by the server",
            format_args!("ID{id}").bold()
        );
        _ = master
            .send(Command::Disconnect.identified_by(
                semaphore.clone().acquire_owned().await.unwrap(),
                id,
            ))
            .await;
    }
}
