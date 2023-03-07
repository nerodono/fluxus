use std::{
    net::SocketAddr,
    sync::Arc,
};

use galaxy_net::{
    raw::packet_type::{
        PacketFlags,
        PacketType,
    },
    reader::{
        GalaxyReader,
        RawReader,
        ReadExt,
        ReadResult,
    },
    schemas::{
        Permissions,
        PingResponseDescriptor,
    },
    shrinker::interface::{
        Compressor,
        Decompressor,
    },
    writer::{
        GalaxyWriter,
        WriteExt,
    },
};
use tokio::select;

use super::client::Client;
use crate::{
    config::Config,
    utils::extract_used_compression,
};

async fn dispatch_network<W, R, C, D>(
    (pkt_type, pkt_flags): (PacketType, PacketFlags),
    client: &mut Client,
    reader: RawReader<'_, R, D>,
    writer: &mut GalaxyWriter<W, C>,
    address: SocketAddr,
    config: &Arc<Config>,
) -> ReadResult<()>
where
    W: WriteExt,
    R: ReadExt,
    C: Compressor,
    D: Decompressor,
{
    match pkt_type {
        PacketType::Ping => {
            let (numeric_method, selected) =
                extract_used_compression(&config.compression);
            GalaxyWriter::server(writer.raw())
                .write_ping(PingResponseDescriptor {
                    server_name: &config.server.name,
                    read_buffer: config
                        .server
                        .bufferization
                        .read
                        .get(),
                    compression_threshold: selected.threshold.get()
                        as u16,
                    compression_method: numeric_method,
                    compression_level: selected.level.get() as u8,
                })
                .await?;
        }

        _ => todo!(),
    }

    Ok(())
}

//

pub async fn run_tcp_listener<W, R, C, D>(
    mut reader: GalaxyReader<R, D>,
    mut writer: GalaxyWriter<W, C>,
    address: SocketAddr,
    config: Arc<Config>,
) -> ReadResult<()>
where
    R: ReadExt,
    W: WriteExt,
    C: Compressor,
    D: Decompressor,
{
    let mut client = Client {
        perms: Permissions::from(&config.permissions.just_connected),
        tcp_server: None,
    };

    loop {
        let mut raw = reader.raw();
        select! {
            pkt_type = raw.read_packet_type() => {
                dispatch_network(
                    pkt_type?,
                    &mut client,
                    raw,
                    &mut writer,
                    address,
                    &config
                )
                .await?;
            }
        }
    }
}
