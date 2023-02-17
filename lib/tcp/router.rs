use std::{
    io,
    net::SocketAddr,
};

use mid_net::{
    prelude::{
        impl_::interface::{
            ICompressor,
            IDecompressor,
        },
        *,
    },
    proto::PacketType,
};

use super::{
    handlers::{
        message_types::MasterMessage,
        network,
    },
    state::State,
};
use crate::config::base::Config;

/// Routes every incoming packet or message to the specific
/// function.
pub async fn run_tcp_packet_router<R, W, D, C>(
    mut reader: MidReader<R, D>,
    mut writer: MidWriter<W, C>,
    address: SocketAddr,
) -> io::Result<()>
where
    R: ReaderUnderlyingExt,
    W: WriterUnderlyingExt,
    C: ICompressor,
    D: IDecompressor,
{
    let config = Config::instance();
    let mut state = State::new(&config.permissions.connect);

    loop {
        tokio::select! {
            message = state.recv_master_message() => {
                match message {
                    Ok(message) => {
                        match message {
                            MasterMessage::Forward { id, data } => {
                                todo!()
                            }

                            MasterMessage::Connected { id, tx } => {
                                todo!()
                            }

                            MasterMessage::Disconnected { id } => {
                                todo!()
                            }

                            MasterMessage::Shutdown => {
                                todo!()
                            }
                        }
                    }

                    Err(e) => {
                        tracing::error!(
                            error = %e,
                            "Failed to recv master message: {e}"
                        );

                        break Ok(());
                    }
                }
            }

            packed = reader.read_raw_packet_type() => {
                let (packet_type, packet_flags) = packed?;
                if let Ok(packet_type) = PacketType::try_from(packet_type) {
                    match packet_type {
                        p @ (PacketType::Connect
                        | PacketType::Failure
                        | PacketType::UpdateRights) => {
                            network::on_unexpected(&mut writer, &address, p).await
                        }

                        PacketType::CreateServer => {
                            network::on_create_server(
                                &mut writer,
                                &mut reader,
                                &mut state,
                                &address,
                                packet_flags,
                            )
                            .await
                        }

                        PacketType::Authorize => {
                            network::on_authorize(
                                &mut writer,
                                &mut reader,
                                &mut state,
                                &address,
                                &config.permissions.universal_password,
                                &config.server.universal_password,
                            )
                            .await
                        }

                        PacketType::Forward => {
                            network::on_forward(
                                &mut writer,
                                &mut reader,
                                &state,
                                &address,
                                packet_flags,
                                DecompressionConstraint::Max(
                                    config.server.bufferization.read,
                                ),
                            )
                            .await
                        }
                        PacketType::Disconnect => {
                            network::on_disconnect(
                                &mut writer,
                                &mut reader,
                                &mut state,
                                packet_flags,
                            )
                            .await
                        }

                        PacketType::Ping => network::on_ping(&mut writer, config).await,
                    }?
                } else {
                    network::on_unknown_packet(
                        &mut writer,
                        address,
                        packet_type,
                        packet_flags,
                    )
                    .await?;
                }
            }
        }
    }
}
