use std::{
    net::SocketAddr,
    sync::Arc,
};

use galaxy_network::{
    raw::{
        ErrorCode,
        PacketType,
    },
    reader::{
        GalaxyReader,
        ReadResult,
    },
    writer::GalaxyWriter,
};
use owo_colors::OwoColorize;
use tokio::{
    io::BufReader,
    net::TcpStream,
};

use crate::{
    config::Config,
    data::{
        idpool::IdPool,
        user::User,
    },
    events,
    protocol::galaxy::network_events,
    utils,
};

pub async fn handle_client<IdPoolFn>(
    config: Arc<Config>,
    mut stream: TcpStream,
    address: SocketAddr,
    id_pool_factory: IdPoolFn,
) -> ReadResult<()>
where
    IdPoolFn: Fn() -> IdPool,
{
    let read_buffer_capacity = config.server.buffering.read;
    let (compressor, decompressor) =
        utils::compression::create_compressor_decompressor(
            &config.compression,
        );
    let (reader, writer) = stream.split();
    let (mut reader, mut writer) = (
        GalaxyReader::new(
            BufReader::with_capacity(
                read_buffer_capacity.get() + 10,
                reader,
            ),
            decompressor,
        ),
        GalaxyWriter::new(writer, compressor),
    );
    let mut user =
        User::new(id_pool_factory(), config.rights.on_connect.to_bits());

    loop {
        let packet_type;
        tokio::select! {
            pkt_ty = reader.read_packet_type() => {
                packet_type = pkt_ty?;
            }

            command = user.proxy.recv_chan.recv() => {
                let Some(command) = command else {
                    tracing::error!(
                        "Failed to pull commands, closing connection for {}...",
                        address.bold()
                    );
                    break Ok(());
                };

                events::dispatch::dispatch_command(
                    &mut writer,
                    command,
                    &mut user
                ).await?;
                continue;
            }
        }

        match packet_type.type_ {
            PacketType::Ping => {
                network_events::ping(&mut writer, address, &config)
                    .await?;
            }

            PacketType::AuthorizePassword => {
                network_events::authorize_password(
                    &mut writer,
                    &mut reader,
                    address,
                    &config,
                    &mut user,
                )
                .await?;
            }

            PacketType::CreateServer => {
                network_events::create_server(
                    &mut writer,
                    &mut reader,
                    address,
                    &mut user,
                    packet_type.flags,
                )
                .await?;
            }

            PacketType::Forward => {
                todo!();
            }

            PacketType::Disconnect => {
                todo!();
            }

            otherwise => {
                tracing::error!(
                    "{} Sent unsupported packet: {otherwise:?}",
                    address.bold()
                );
                writer
                    .server()
                    .write_error(ErrorCode::Unsupported)
                    .await?;
            }
        }
    }
}
