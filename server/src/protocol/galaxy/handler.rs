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
use tokio::{
    io::BufReader,
    net::TcpStream,
};

use crate::{
    config::Config,
    data::idpool::IdPool,
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
        utils::create_compressor_decompressor(&config.compression);
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

    todo!()
}
