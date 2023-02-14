use std::io;

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

use super::handlers::network;
use crate::config::base::Config;

/// Routes every incoming packet or message to the specific
/// function.
pub async fn run_tcp_packet_router<R, W, D, C>(
    mut reader: MidReader<R, D>,
    mut writer: MidWriter<W, C>,
) -> io::Result<()>
where
    R: ReaderUnderlyingExt,
    W: WriterUnderlyingExt,
    C: ICompressor,
    D: IDecompressor,
{
    let config = Config::instance();
    loop {
        let (packet_type, packet_flags) =
            reader.read_raw_packet_type().await?;
        if let Ok(packet_type) = PacketType::try_from(packet_type) {
            match packet_type {
                PacketType::Connect => todo!(),
                PacketType::Disconnect => todo!(),
                PacketType::Failure => todo!(),
                PacketType::Forward => todo!(),
                PacketType::Ping => {
                    network::on_ping(&mut writer, config).await
                }
            }?
        } else {
            network::on_unknown_packet(
                &mut writer,
                packet_type,
                packet_flags,
            )
            .await?;
        }
    }
}
