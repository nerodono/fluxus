use std::{
    net::SocketAddr,
    sync::Arc,
};

use galaxy_network::{
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
        command::erased::ErasedCommand,
        idpool::IdPool,
        user::User,
    },
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

                match command {
                    ErasedCommand::Tcp(tcp_command) => {
                        todo!()
                    }
                }
                continue;
            }
        }

        match packet_type.type_ {
            _ => todo!(),
        }
    }
}
