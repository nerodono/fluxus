use std::{
    io,
    sync::Arc,
};

use tokio::{
    io::{
        AsyncReadExt,
        AsyncWriteExt,
    },
    net::TcpStream,
    sync::{
        mpsc,
        Notify,
    },
};

use crate::protocols::tcp_flux::events::{
    flow::FlowEvent,
    master::FlowMasterCommand,
};

pub async fn run_connection_handler(
    notify: Arc<Notify>,
    buffer_size: usize,
    mut stream: TcpStream,
    master_push: &mpsc::Sender<FlowEvent>,
    mut flow_rx: mpsc::Receiver<FlowMasterCommand>,
) -> io::Result<()> {
    // Wait until handshake is performed
    notify.notified().await;

    let mut buffer = vec![0; buffer_size];
    loop {
        tokio::select! {
            command = flow_rx.recv() => {
                let Some(command) = command else {
                    return Ok(());
                };
                match command {
                    FlowMasterCommand::Forward { buf } => {
                        stream.write_all(&buf).await?;
                    }

                    FlowMasterCommand::Close => {
                        return Ok(());
                    }
                }
            }

            read_result = stream.read(&mut buffer) => {
                let read @ 1.. = read_result? else {
                    return Ok(());
                };

                if master_push.send(
                    FlowEvent::Wrote { buf: Vec::from(&buffer[..read]) }
                ).await.is_err() {
                    return Ok(());
                }
            }
        }
    }
}
