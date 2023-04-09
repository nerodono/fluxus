use std::{
    borrow::Cow,
    net::SocketAddr,
    num::{
        NonZeroU16,
        NonZeroU8,
        NonZeroUsize,
    },
    process,
};

use galaxy_network::{
    descriptors::{
        CompressionDescriptor,
        CreateServerResponseDescriptor,
        PingResponseDescriptor,
    },
    error::ReadError,
    raw::{
        CompressionAlgorithm,
        PacketFlags,
        PacketType,
        Protocol,
    },
    reader::{
        GalaxyReader,
        Read,
        ReadResult,
    },
    shrinker::{
        interface::{
            CompressorStub,
            DecompressorStub,
        },
        zstd::{
            ZStdCctx,
            ZStdDctx,
        },
    },
    writer::GalaxyWriter,
};
use humansize::{
    format_size,
    DECIMAL,
};
use owo_colors::OwoColorize;
use tokio::net::TcpStream;

pub async fn run_work(
    mut stream: TcpStream,
    password: Option<&str>,
    server_address: &str,
    local: SocketAddr,
    port: Option<NonZeroU16>,
) -> eyre::Result<()> {
    let (reader, writer) = stream.split();
    let (mut reader, mut writer) = (
        GalaxyReader::new(reader, DecompressorStub),
        GalaxyWriter::new(writer, CompressorStub),
    );
    writer.client().write_ping().await?;
    expect_packet(&mut reader, PacketType::Ping).await?;
    let ping = read_ping(&mut reader).await?;

    tracing::info!(
        "| {} ({}) settings:",
        ping.server_name.bold(),
        server_address.cyan()
    );
    tracing::info!(
        "| {} {:?} (level {})",
        "Compression:".bold(),
        ping.compression.algorithm,
        ping.compression.level.get()
    );
    tracing::info!(
        "| {} {}",
        "Read buffer capacity:".bold(),
        format_size(ping.buffer_read.get(), DECIMAL)
    );

    if let Some(password) = password {
        writer
            .client()
            .write_password_auth(password)
            .await?;
        expect_packet(&mut reader, PacketType::UpdateRight).await?;

        let rights = reader.read_rights().await?;
        tracing::info!("| Got rights: {:?}", rights.bold());
    }

    writer
        .client()
        .write_server_request(Protocol::Tcp, port)
        .await?;
    let server = expect_server(&mut reader, port).await?;
    let port = server
        .port
        .map(|p| p.to_string())
        .unwrap_or_else(|| {
            String::from("- (Server decided that you know the port)")
        });
    tracing::info!(
        "Started TCP server on {}",
        format_args!(":{port}").bold()
    );

    match ping.compression.algorithm {
        CompressionAlgorithm::ZStd => {
            super::loop_::loop_(
                GalaxyReader::new(reader.into_inner().0, ZStdDctx::new()),
                GalaxyWriter::new(
                    writer.into_inner().0,
                    ZStdCctx::new(ping.compression.level),
                ),
                ping.buffer_read.get(),
                local,
            )
            .await
        }
    }
}

// FIXME: move these helpers to the `galaxy-network` crate?

async fn expect_server<R: Read, D>(
    reader: &mut GalaxyReader<R, D>,
    port: Option<NonZeroU16>,
) -> eyre::Result<CreateServerResponseDescriptor> {
    let flags = expect_packet(reader, PacketType::CreateServer).await?;
    Ok(if flags.intersects(PacketFlags::COMPRESSED) {
        CreateServerResponseDescriptor { port }
    } else {
        let port = reader.read_u16().await?;
        CreateServerResponseDescriptor {
            port: NonZeroU16::new(port),
        }
    })
}

async fn read_ping<R: Read, D>(
    reader: &mut GalaxyReader<R, D>,
) -> ReadResult<PingResponseDescriptor<'static>> {
    let compression_algorithm =
        reader.read_compression_algorithm().await?;
    let repr = reader.read_u8().await?;
    let compression_level = NonZeroU8::new(repr)
        .ok_or(ReadError::InvalidCompressionLevel(repr))?;

    let repr = reader.read_u16().await?;
    let buffer_read = NonZeroUsize::new(repr as usize)
        .ok_or(ReadError::InvalidReadBuffer(repr))?;

    let server_name = reader.read_string_prefixed().await?;

    Ok(PingResponseDescriptor {
        compression: CompressionDescriptor {
            algorithm: compression_algorithm,
            level: compression_level,
        },

        server_name: Cow::Owned(server_name),
        buffer_read,
    })
}

async fn expect_packet<R: Read, D>(
    reader: &mut GalaxyReader<R, D>,
    pkt_ty: PacketType,
) -> eyre::Result<PacketFlags> {
    let packet = reader.read_packet_type().await?;
    if packet.type_ != pkt_ty {
        if packet.type_ == PacketType::Error {
            let code = reader.read_error_code().await?;
            tracing::error!(
                "Expected packet {:?}, got error: {code}",
                pkt_ty.bold()
            );
        } else {
            tracing::error!(
                "Unexpected packet {:?}, expected: {:?}",
                packet.type_.bold(),
                pkt_ty.bold()
            );
        }

        process::exit(1);
    }

    Ok(packet.flags)
}
