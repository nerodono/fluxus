use std::io;

use mid_net::prelude::{
    impl_::polymorphic::{
        PolyCompressor,
        PolyDecompressor,
    },
    *,
};
use tokio::net::TcpListener;

use crate::{
    config::base::{
        Config,
        ProtocolCompressionCfg,
    },
    tcp::router,
};

/// Creates polymorphic compressor/decompressor with
/// settings from the config
fn create_compdecomp_pair(
    cfg: &ProtocolCompressionCfg,
) -> (PolyCompressor, PolyDecompressor) {
    match cfg.algorithm {
        CompressionAlgorithm::ZStd => (
            PolyCompressor::zstd(
                cfg.level
                    .get()
                    .try_into()
                    .expect("Too long compression level"),
            ),
            PolyDecompressor::zstd(),
        ),
        CompressionAlgorithm::Deflate => {
            unimplemented!()
        }
    }
}

/// Main server function. Listens for the incoming
/// connections from the provided [`TcpListener`] and spawns
/// `packet router` task.
pub async fn run_tcp_listener(listener: TcpListener) -> io::Result<()> {
    let config = Config::instance();
    let listening_address = listener.local_addr()?;
    tracing::info!(%listening_address, "Server is started");

    loop {
        let (mut stream, address) = listener.accept().await?;
        tracing::info!(%address, "Connected to the server");

        tokio::spawn(async move {
            let (compressor, decompressor) =
                create_compdecomp_pair(&config.compression.tcp);
            let (reader, writer) = stream.as_buffered_rw_handles(
                compressor,
                decompressor,
                config.server.bufferization.read,
            );

            router::run_tcp_packet_router(reader, writer)
                .await
                .unwrap_or(());
        });
    }
}
