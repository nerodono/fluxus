use std::collections::HashMap;

use galaxy_network::{
    raw::{
        PacketFlags,
        PacketType,
    },
    reader::{
        GalaxyReader,
        Read,
    },
    shrinker::interface::{
        Compressor,
        Decompressor,
    },
    writer::{
        GalaxyWriter,
        Write,
    },
};
use owo_colors::OwoColorize;
use rustc_hash::FxHashMap;
use tokio::{
    net::TcpStream,
    sync::mpsc,
};

use crate::{
    command::{
        Command,
        IdentifiedCommand,
    },
    connection_slave::run_slave,
};

pub struct Connection<R, D, W, C> {
    pub reader: GalaxyReader<R, D>,
    pub writer: GalaxyWriter<W, C>,
    pub local: String,
    pub buffer_size: usize,

    map: FxHashMap<u16, mpsc::UnboundedSender<Command>>,
    tx: mpsc::UnboundedSender<IdentifiedCommand>,
    rx: mpsc::UnboundedReceiver<IdentifiedCommand>,
}

impl<R, D, W, C> Connection<R, D, W, C>
where
    R: Read,
    W: Write,
    C: Compressor,
    D: Decompressor,
{
    async fn handle_error(&mut self) -> eyre::Result<()> {
        let error_code = self.reader.read_error_code().await?;
        tracing::error!("Got error: {error_code}");

        Ok(())
    }

    async fn handle_disconnect(
        &mut self,
        flags: PacketFlags,
    ) -> eyre::Result<()> {
        let client_id = self.reader.read_client_id(flags).await?;
        let channel = self.map.remove(&client_id).unwrap();
        _ = channel.send(Command::Disconnect);

        tracing::info!("Client {client_id} is disconnected");
        Ok(())
    }

    async fn handle_connect(
        &mut self,
        flags: PacketFlags,
    ) -> eyre::Result<()> {
        let client_id = self.reader.read_client_id(flags).await?;
        let (tx, rx) = mpsc::unbounded_channel();
        self.map.insert(client_id, tx);

        let stream = match TcpStream::connect(&self.local).await {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to connect to the local server: {e}");
                self.writer.write_disconnected(client_id).await?;
                return Ok(());
            }
        };

        tokio::spawn(run_slave(
            client_id,
            stream,
            self.tx.clone(),
            rx,
            self.buffer_size,
        ));

        Ok(())
    }

    async fn handle_forward(
        &mut self,
        flags: PacketFlags,
    ) -> eyre::Result<()> {
        let client_id = self.reader.read_client_id(flags).await?;
        let length = self.reader.read_forward_length(flags).await? as usize;
        let buffer = self
            .reader
            .try_read_forward_buffer(length, |_| true, flags)
            .await?;

        let channel = self.map.get(&client_id).unwrap();
        _ = channel.send(Command::Forward { buffer });

        Ok(())
    }

    async fn handle_command(
        &mut self,
        IdentifiedCommand { id, command }: IdentifiedCommand,
    ) -> eyre::Result<()> {
        match command {
            Command::Forward { buffer } => {
                self.writer
                    .write_forward(id, &buffer, buffer.len() >= 64)
                    .await?;
            }

            Command::Disconnect => {
                tracing::info!(
                    "Client {} was disconnected by the server",
                    id.bold()
                );
                self.map.remove(&id);
                self.writer.write_disconnected(id).await?;
            }
        }
        Ok(())
    }

    pub async fn run(&mut self) -> eyre::Result<()> {
        loop {
            let packet;
            tokio::select! {
                command = self.rx.recv() => {
                    self.handle_command(command.unwrap()).await?;
                    continue;
                }

                pkt = self.reader.read_packet_type() => {
                    packet = pkt?;
                }
            }

            match packet.type_ {
                PacketType::Connect => {
                    self.handle_connect(packet.flags).await
                }
                PacketType::Disconnect => {
                    self.handle_disconnect(packet.flags).await
                }
                PacketType::Forward => {
                    self.handle_forward(packet.flags).await
                }

                PacketType::Error => self.handle_error().await,

                pkt => {
                    panic!("Unexpected packet type: {pkt:?}")
                }
            }?;
        }
    }
}

impl<R, D, W, C> Connection<R, D, W, C> {
    pub fn new(
        local: String,
        buffer_size: usize,

        reader: GalaxyReader<R, D>,
        writer: GalaxyWriter<W, C>,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            reader,
            writer,
            local,
            buffer_size,
            map: HashMap::default(),
            tx,
            rx,
        }
    }
}