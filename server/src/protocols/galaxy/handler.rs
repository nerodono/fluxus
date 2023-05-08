use std::{
    net::SocketAddr,
    sync::Arc,
};

use galaxy_network::{
    raw::PacketType,
    reader::GalaxyReader,
    writer::GalaxyWriter,
};
use owo_colors::OwoColorize;
use tokio::{
    io::BufReader,
    net::TcpStream,
};

use super::connection::Connection;
use crate::{
    communication::dispatcher::CommandDispatcher,
    config::Config,
    data::user::User,
    error::{
        ProcessError,
        ProcessResult,
    },
    utils::{
        self,
        feature_gate::FeatureGate,
    },
};

pub async fn handle_connection(
    mut stream: TcpStream,
    address: SocketAddr,
    config: Arc<Config>,
    gate: FeatureGate,
) -> ProcessResult<()> {
    let (raw_reader, raw_writer) = stream.split();
    let (compressor, decompressor) =
        utils::compression::create_compressor_decompressor(
            &config.compression,
        );
    let (reader, writer) = (
        GalaxyReader::new(
            BufReader::with_capacity(
                config.server.buffering.read.get(),
                raw_reader,
            ),
            decompressor,
        ),
        GalaxyWriter::new(raw_writer, compressor),
    );
    let mut connection = Connection {
        user: User::new(config.rights.on_connect.to_bits()),

        reader,
        writer,
        address,
        config: &config,
        gate,
    };
    let dispatcher =
        CommandDispatcher::new(address, config.compression.threshold);

    loop {
        let packet;
        tokio::select! {
            command = connection.user.recv_command() => {
                let Some(command) = command else {
                    tracing::error!("{} failed to recv master command, disconnecting...", address.bold());
                    break;
                };
                let reset_proxy = {
                    let proxy = unsafe { connection.user.proxy.as_mut().unwrap_unchecked() };
                    dispatcher.dispatch(&mut connection.writer, proxy, command).await?
                };

                if reset_proxy {
                    connection.user.proxy = None;
                    connection.server_stopped().await?;
                }

                continue;
            }

            pkt = connection.reader.read_packet_type() => {
                packet = pkt?;
            }
        }

        let processing_result = match packet.type_ {
            PacketType::Ping => connection.ping().await,

            PacketType::CreateServer => {
                connection.create_server(packet.flags).await
            }

            PacketType::Forward => connection.forward(packet.flags).await,

            PacketType::Disconnect => {
                connection.disconnected(packet.flags).await
            }

            PacketType::AuthorizePassword => {
                connection.authorize_password().await
            }

            pkt_ty => {
                tracing::error!(
                    "{} sent unknown packet ({pkt_ty:?}), disconnecting...",
                    connection.address.bold()
                );
                break;
            }
        };

        match processing_result {
            Ok(()) => {}
            Err(ProcessError::NonCritical(non_critical_error)) => {
                tracing::error!(
                    "{} got non-critical error: {non_critical_error}",
                    address.bold()
                );
                _ = connection
                    .writer
                    .server()
                    .write_error(non_critical_error.into())
                    .await;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(())
}
