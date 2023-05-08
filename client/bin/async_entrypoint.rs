use fluxus::{
    cli_args::{
        CliArgs,
        CliSub,
    },
    connection::Connection,
    utils::compression::create_compressor_decompressor,
};
use galaxy_network::{
    descriptors::CreateServerRequestDescriptor,
    raw::{
        PacketFlags,
        PacketType,
    },
    reader::{
        GalaxyReader,
        Read,
        ReadResult,
    },
    shrinker::interface::{
        CompressorStub,
        DecompressorStub,
    },
    writer::GalaxyWriter,
};
use humansize::{
    format_size,
    BINARY,
};
use owo_colors::OwoColorize;
use tokio::{
    io::BufReader,
    net::TcpStream,
};

pub async fn async_main(args: CliArgs) -> eyre::Result<()> {
    tracing::info!(
        "Connecting to the fluxus master server ({})...",
        args.remote.bold()
    );
    let mut stream = TcpStream::connect(&args.remote).await?;
    let (raw_reader, raw_writer) = stream.split();
    let (mut reader, mut writer) = (
        GalaxyReader::new(raw_reader, DecompressorStub),
        GalaxyWriter::new(raw_writer, CompressorStub),
    );
    if let Some(ref password) = args.password {
        writer
            .client()
            .write_password_auth(password)
            .await?;
        expect_packet(&mut reader, PacketType::UpdateRights).await?;
        let rights = reader.read_rights().await?;

        tracing::info!("Got the following rights: {rights:?}");
    }

    writer.client().write_ping().await?;
    expect_packet(&mut reader, PacketType::Ping).await?;
    let ping = reader.client().read_ping().await?;

    tracing::info!("+--");
    tracing::info!("| Connected to the server {}", ping.server_name.bold());
    tracing::info!(
        "| Compression algorithm: {:?} (level = {})",
        ping.compression.algorithm.bold(),
        ping.compression.level.bold(),
    );
    tracing::info!(
        "| Read buffer size: {}",
        format_size(ping.buffer_read.get(), BINARY)
    );

    tracing::info!("+--");

    let (compressor, decompressor) =
        create_compressor_decompressor(&ping.compression);
    let ((reader, _), (writer, _)) =
        (reader.into_inner(), writer.into_inner());
    let (mut reader, mut writer) = (
        GalaxyReader::new(
            BufReader::with_capacity(ping.buffer_read.get() + 5, reader),
            decompressor,
        ),
        GalaxyWriter::new(writer, compressor),
    );

    let local_addr = match args.sub {
        CliSub::Http { local, domain } => {
            writer
                .client()
                .write_server_request(CreateServerRequestDescriptor::Http {
                    endpoint: domain.as_bytes(),
                })
                .await?;
            let flags =
                expect_packet(&mut reader, PacketType::CreateServer).await?;

            let endpoint = if flags.contains(PacketFlags::SHORT) {
                domain
            } else {
                reader.read_string_prefixed().await?
            };

            tracing::info!("Bound HTTP server on domain {}", endpoint.bold());
            local
        }

        CliSub::Tcp { local, port } => {
            writer
                .client()
                .write_server_request(CreateServerRequestDescriptor::Tcp {
                    port,
                })
                .await?;
            let flags =
                expect_packet(&mut reader, PacketType::CreateServer).await?;
            let port_string = if flags.contains(PacketFlags::COMPRESSED) {
                port.map_or_else(
                    || "(Server thinks that you should know it)".to_owned(),
                    |p| p.to_string(),
                )
            } else {
                reader.read_u16().await?.to_string()
            };

            tracing::info!(
                "Bound TCP server on {}",
                format_args!(":{port_string}").bold()
            );

            local
        }
    };

    let mut connection =
        Connection::new(local_addr, ping.buffer_read.get(), reader, writer);
    connection.run().await
}

async fn expect_packet<R, C>(
    reader: &mut GalaxyReader<R, C>,
    ty: PacketType,
) -> ReadResult<PacketFlags>
where
    R: Read,
{
    let packet = reader.read_packet_type().await?;
    if packet.type_ == ty {
        Ok(packet.flags)
    } else {
        match packet.type_ {
            // Special case for handling errors.
            // It would be handy to display happened error instead of just
            // falling with abstract "Error happened"
            PacketType::Error => {
                let error_code = reader.read_error_code().await?;
                tracing::error!(
                    "Expected packet {ty:?}, got error instead: {error_code}"
                );
            }

            p => {
                tracing::error!("Expected packet {ty:?}, got {p:?} instead");
            }
        }
        panic!()
    }
}
