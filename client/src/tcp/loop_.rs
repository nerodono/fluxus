use std::net::SocketAddr;

use galaxy_network::{
    raw::PacketType,
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

use super::{
    command::MasterCommand,
    network,
};
use crate::tcp::{
    command,
    server::TcpRemoteServer,
};

pub async fn loop_<R, D, W, C>(
    mut reader: GalaxyReader<R, D>,
    mut writer: GalaxyWriter<W, C>,
    buffer_size: usize,
    connect_to: SocketAddr,
) -> eyre::Result<()>
where
    R: Read,
    W: Write,
    D: Decompressor,
    C: Compressor,
{
    let mut server = TcpRemoteServer::default();
    loop {
        tokio::select! {
            command = server.chan_rx.recv() => {
                let command = command.expect(
                    "Failed to get command from the queue");
                match command {
                    MasterCommand::Forward { id, buffer } => {
                        command::forward(&mut writer, id, buffer).await?;
                    }

                    MasterCommand::Disconnect { id } => {
                        command::disconnect(
                            &mut writer, &mut server, id).await?;
                    }
                }
            }

            pkt = reader.read_packet_type() => {
                let packet = pkt?;
                match packet.type_ {
                    PacketType::Forward => {
                        network::forward(
                            &mut reader,
                            &mut writer,
                            packet.flags,
                            &mut server
                        )
                        .await?;
                    }

                    PacketType::Disconnect => {
                        network::disconnect(
                            &mut reader,
                            &mut server,
                            packet.flags
                        )
                        .await?;
                    }

                    PacketType::Connect => {
                        network::connect(
                            buffer_size,
                            connect_to,
                            &mut reader,
                            packet.flags,
                            &mut server
                        )
                        .await?;
                    }

                    PacketType::Error => {
                        let error_code = reader.read_error_code().await?;
                        tracing::error!("Got error: {}", error_code.bold().red());
                    }

                    p => {
                        tracing::error!(
                            "Unexpected packet {:?} in stream",
                            p.bold()
                        );
                    }
                }
            }
        }
    }
}
