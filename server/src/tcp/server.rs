use std::{
    net::SocketAddr,
    sync::Arc,
};

use galaxy_network::{
    error::ReadError,
    raw::{
        ErrorCode,
        PacketType,
    },
    reader::GalaxyReader,
    shrinker::interface::{
        CompressorStub,
        DecompressorStub,
    },
    writer::GalaxyWriter,
};
use owo_colors::OwoColorize;
use tokio::{
    io::BufReader,
    net::{
        TcpListener,
        TcpStream,
    },
};

use crate::{
    config::Config,
    tcp::network,
};

async fn listen_to_stream(
    config: Arc<Config>,
    mut stream: TcpStream,
    address: SocketAddr,
) -> eyre::Result<()> {
    let (reader, writer) = stream.split();
    let (mut reader, mut writer) = (
        GalaxyReader::new(
            BufReader::with_capacity(
                config.server.buffering.read.get(),
                reader,
            ),
            DecompressorStub,
        ),
        GalaxyWriter::new(writer, CompressorStub),
    );

    loop {
        let packet = match reader.read_packet_type().await {
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
            PacketType::Ping => {
                tracing::info!("{} Ping ", address.bold());
                network::ping(&mut writer, &config).await?;
            }

            PacketType::CreateServer => {}

            #[allow(unused_parens)]
            u @ (PacketType::Error) => {
                tracing::error!(
                    "{} Sent unsupported packet: {u:?}",
                    address.bold()
                );
            }
        }
    }
}

pub async fn run_server(config: Arc<Config>) -> eyre::Result<()> {
    let listener = TcpListener::bind(&config.server.listen).await?;
    let address = listener.local_addr()?;

    tracing::info!("Started TCP server on {}", address.bold());
    loop {
        let (stream, connected_address) = listener.accept().await?;
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