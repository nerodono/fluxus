use std::{
    net::SocketAddr,
    sync::Arc,
};

use galaxy_net::{
    raw::related::CompressionMethod,
    reader::{
        GalaxyReader,
        ReadResult,
    },
    shrinker::backends::polymorphic::{
        PolymorphicCctx,
        PolymorphicDctx,
    },
    writer::GalaxyWriter,
};
use tokio::net::{
    TcpListener,
    TcpStream,
};

use super::listener;
use crate::config::{
    CompressionEntry,
    Config,
};

fn create_compressor_decompressor(
    cfg: &CompressionEntry,
) -> (PolymorphicCctx, PolymorphicDctx) {
    match cfg.use_ {
        CompressionMethod::ZStd => (
            PolymorphicCctx::zstd(cfg.zstd.level),
            PolymorphicDctx::zstd(),
        ),
    }
}

async fn spawner(
    mut stream: TcpStream,
    address: SocketAddr,
    config: Arc<Config>,
) -> ReadResult<()> {
    let (raw_reader, raw_writer) = stream.split();

    let (compressor, decompressor) =
        create_compressor_decompressor(&config.compression);
    let (reader, writer) = (
        GalaxyReader::new_buffered(
            raw_reader,
            decompressor,
            config.server.bufferization.read.get(),
        ),
        GalaxyWriter::new(raw_writer, compressor),
    );

    listener::run_tcp_listener(reader, writer, address, config).await
}

pub async fn run_tcp_spawner(
    listener: TcpListener,
    config: Config,
) -> ReadResult<()> {
    let config = Arc::new(config);
    loop {
        let (stream, address) = listener.accept().await?;
        tracing::info!(%address, "New connection to main server");

        tokio::spawn(spawner(stream, address, Arc::clone(&config)));
    }
}
