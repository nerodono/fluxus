use std::{
    net::SocketAddr,
    sync::Arc,
};

use galaxy_network::{
    error::ReadError,
    raw::{
        CompressionAlgorithm,
        ErrorCode,
        PacketType,
    },
    reader::GalaxyReader,
    shrinker::zstd::{
        ZStdCctx,
        ZStdDctx,
    },
    writer::GalaxyWriter,
};
use idpool::flat::FlatIdPool;
use owo_colors::OwoColorize;
use tokio::{
    io::BufReader,
    net::{
        TcpListener,
        TcpStream,
    },
    sync::Mutex,
};

use crate::{
    config::{
        CompressionConfig,
        Config,
    },
    logic::{
        command::MasterCommand,
        tcp_server::TcpIdPool,
        user::User,
    },
    tcp::{
        command,
        network,
    },
};

fn create_id_pool() -> TcpIdPool {
    Arc::new(Mutex::new(FlatIdPool::new(0_u16)))
}

// FIXME: Polymorphic compression support
fn create_compressor_decompressor(
    cfg: &CompressionConfig,
) -> (ZStdCctx, ZStdDctx) {
    match cfg.algorithm {
        CompressionAlgorithm::ZStd => {
            (ZStdCctx::new(cfg.level), ZStdDctx::new())
        }
    }
}

async fn listen_to_stream(
    config: Arc<Config>,
    mut stream: TcpStream,
    address: SocketAddr,
) -> eyre::Result<()> {
    let (reader, writer) = stream.split();
    let (compressor, decompressor) =
        create_compressor_decompressor(&config.compression);
    let (mut reader, mut writer) = (
        GalaxyReader::new(
            BufReader::with_capacity(
                config.server.buffering.read.get(),
                reader,
            ),
            decompressor,
        ),
        GalaxyWriter::new(writer, compressor),
    );
    let mut user = User::new(config.rights.on_connect.to_bits());
    let mut running = true;

    while running {
        let packet;
        tokio::select! {
            pkt = reader.read_packet_type() => {
                packet = pkt;
            }

            command = user.recv_command() => {
                let Some(command) = command else {
                    tracing::error!("{} Unexpected behavior of channel, report this on github", address.bold());
                    break;
                };

                let server = user.tcp_proxy.as_mut().unwrap();
                match command {
                    MasterCommand::Connected { id, channel } => {
                        command::connected(&mut writer, server, id, channel).await?;
                    }

                    MasterCommand::Disconnected { id } => {
                        command::disconnected(&mut writer, server, id).await?;
                    }

                    MasterCommand::Forward { id, buffer } => {
                        command::forward(&mut writer, id, buffer, &config).await?;
                    }

                    MasterCommand::Stopped => {
                        user.tcp_proxy = None;
                    }
                }

                continue;
            }
        }

        let packet = match packet {
            Ok(p) => p,
            Err(ReadError::UnknownPacket) => {
                tracing::error!("{} Sent unknown packet", address.bold());
                let _ = writer
                    .server()
                    .write_error(ErrorCode::UnknownCommand)
                    .await;
                return Err(ReadError::UnknownPacket.into());
            }

            Err(e) => return Err(e.into()),
        };

        match packet.type_ {
            PacketType::AuthorizePassword => {
                network::authorize_password(
                    address,
                    &mut writer,
                    &mut reader,
                    &mut user,
                    &config,
                )
                .await?;
            }
            PacketType::Forward => {
                network::forward(
                    address,
                    &mut writer,
                    &mut reader,
                    packet,
                    &mut user,
                    &config,
                    &mut running,
                )
                .await?;
            }

            PacketType::Disconnect => {
                network::disconnect(
                    address,
                    &mut writer,
                    &mut reader,
                    packet,
                    &mut user,
                )
                .await?;
            }

            PacketType::Ping => {
                tracing::info!("{} Ping ", address.bold());
                network::ping(&mut writer, &config).await?;
            }

            PacketType::CreateServer => {
                network::server_request(
                    address,
                    &mut writer,
                    &mut reader,
                    packet,
                    &mut user,
                    &mut create_id_pool,
                )
                .await?;
            }

            u => {
                tracing::error!(
                    "{} Sent unsupported packet: {u:?}",
                    address.bold()
                );
                writer
                    .server()
                    .write_error(ErrorCode::Unsupported)
                    .await?;
            }
        }
    }

    Ok(())
}

pub async fn run_server(config: Arc<Config>) -> eyre::Result<()> {
    let listener = TcpListener::bind(&config.server.listen).await?;
    let address = listener.local_addr()?;

    tracing::info!("Started TCP server on {}", address.bold());
    loop {
        let (stream, connected_address) = listener.accept().await?;
        if let Err(e) = stream.set_nodelay(true) {
            tracing::error!(
                "Failed to set TCP_NODELAY flag for the {}: {e}",
                connected_address.bold()
            );
            continue;
        }
        tracing::info!(
            "{} Connected to the TCP server",
            connected_address.bold()
        );

        let config = Arc::clone(&config);
        tokio::spawn(async move {
            let _ =
                listen_to_stream(config, stream, connected_address).await;
            tracing::info!(
                "{} Disconnected from the TCP server",
                connected_address.bold()
            );
        });
    }
}
