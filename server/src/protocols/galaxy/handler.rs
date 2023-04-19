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
    reader::{
        GalaxyReader,
        Read,
        ReadResult,
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

use super::events;
use crate::{
    config::Config,
    data::{
        id_pool::IdPoolImpl,
        user::User,
    },
    events::dispatcher::dispatch_command,
    utils,
};

pub async fn handle_connection<R, D, W, C>(
    mut reader: GalaxyReader<R, D>,
    mut writer: GalaxyWriter<W, C>,
    config: Arc<Config>,
    address: SocketAddr,
    id_pool_factory: impl Fn() -> IdPoolImpl,
) -> ReadResult<()>
where
    R: Read,
    W: Write,
    D: Decompressor,
    C: Compressor,
{
    let mut user = User::new(config.rights.on_connect.to_bits());
    loop {
        let packet;
        tokio::select! {
            command = user.recv_command() => {
                let Some(command) = command else {
                    tracing::error!(
                        "Command channel accidentely closed, closing connection for {}",
                        address.bold()
                    );
                    break;
                };

                dispatch_command(
                    &mut writer,
                    address,
                    &mut user,
                    command,
                    &config
                ).await?;

                continue;
            }

            packet_ty = reader.read_packet_type() => {
                packet = packet_ty?;
            }
        }

        let result = match packet.type_ {
            PacketType::Ping => {
                tracing::info!("{} ping request", address.bold());
                events::ping(&mut writer, &config).await
            }

            PacketType::Forward => {
                events::forward(
                    &mut reader,
                    &mut writer,
                    &mut user,
                    packet.flags,
                    &config,
                )
                .await
            }

            PacketType::Disconnect => {
                events::disconnect(
                    &mut reader,
                    &mut writer,
                    &mut user,
                    packet.flags,
                )
                .await
            }

            PacketType::CreateServer => {
                events::create_server(
                    &mut reader,
                    &mut writer,
                    &mut user,
                    address,
                    packet.flags,
                    &config,
                    &id_pool_factory,
                )
                .await
            }

            PacketType::AuthorizePassword => {
                events::authorize_password(
                    &mut reader,
                    &mut writer,
                    &mut user,
                    &config,
                )
                .await
            }

            p => {
                utils::compiler::cold_fn();
                tracing::error!(
                    "{} sent unsupported packet: {p:?}",
                    address.bold()
                );
                _ = writer
                    .server()
                    .write_error(ErrorCode::Unsupported)
                    .await;
                break;
            }
        };

        match result {
            Err(ReadError::NonCritical) | Ok(()) => {}
            Err(e) => {
                utils::compiler::cold_fn();
                return Err(e);
            }
        }
    }

    Ok(())
}
