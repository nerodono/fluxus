use std::io;

use mid_net::{
    prelude::WriterUnderlyingExt,
    proto::ProtocolError,
    writer::MidWriter,
};

use crate::config::base::Config;

/// Reacts to the `ping`
pub async fn on_ping<W: WriterUnderlyingExt, C>(
    writer: &mut MidWriter<W, C>,
    config: &'static Config,
) -> io::Result<()> {
    writer
        .server()
        .write_ping(
            &config.server.name,
            config.compression.tcp.algorithm,
            config
                .server
                .bufferization
                .read
                .try_into()
                .unwrap_or_else(|e| {
                    let fallback_maximum = u16::MAX;
                    tracing::error!(
                        fallback_maximum,
                        "Failed to write bufferization value ({e}), \
                         writing back fallback maximum"
                    );

                    fallback_maximum
                }),
        )
        .await
}

/// Called when router receives unknown packet type.
/// Basically just logs & writes the error
pub async fn on_unknown_packet<W: WriterUnderlyingExt, C>(
    writer: &mut MidWriter<W, C>,
    packet_type: u8,
    packet_flags: u8,
) -> io::Result<()> {
    tracing::error!(
        packet_type,
        packet_flags,
        "Unknown packet type received"
    );
    writer
        .server()
        .write_failure(ProtocolError::UnknownPacket)
        .await
}
