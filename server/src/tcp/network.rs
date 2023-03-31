use galaxy_network::{
    descriptors::{
        CompressionDescriptor,
        PingResponseDescriptor,
    },
    reader::{
        GalaxyReader,
        Read,
        ReadResult,
    },
    writer::{
        GalaxyWriter,
        Write,
    },
};

use crate::{
    config::Config,
    logic::user::User,
};

pub async fn server_request<W, C, R, D>(
    writer: &mut GalaxyWriter<W, C>,
    reader: &mut GalaxyReader<R, D>,
    config: &Config,
    user: User,
) -> ReadResult<()>
where
    W: Write,
    R: Read,
{
    Ok(())
}

pub async fn ping<W: Write, C>(
    writer: &mut GalaxyWriter<W, C>,
    config: &Config,
) -> ReadResult<()> {
    writer
        .server()
        .write_ping(&PingResponseDescriptor {
            compression: CompressionDescriptor {
                algorithm: config.compression.algorithm,
                level: config.compression.level,
            },

            server_name: &config.server.name,
            buffer_read: config.server.buffering.read,
        })
        .await
        .map_err(Into::into)
}
