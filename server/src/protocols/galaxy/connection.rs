use std::{
    borrow::Cow,
    net::SocketAddr,
    sync::Arc,
};

use galaxy_network::{
    descriptors::{
        CompressionDescriptor,
        PingResponseDescriptor,
    },
    reader::{
        GalaxyReader,
        Read,
    },
    shrinker::interface::{
        Compressor,
        Decompressor,
    },
    writer::{
        GalaxyWriter,
        Write,
    },
};
use owo_colors::OwoColorize;

use crate::{
    config::{
        AuthorizationBackend,
        Config,
    },
    data::user::User,
    error::{
        NonCriticalError,
        ProcessResult,
    },
};

pub struct Connection<R, D, W, C> {
    pub reader: GalaxyReader<R, D>,
    pub writer: GalaxyWriter<W, C>,
    pub address: SocketAddr,
    pub user: User,
    pub config: Arc<Config>,
}

impl<R, D, W, C> Connection<R, D, W, C>
where
    R: Read,
    W: Write,
    D: Decompressor,
    C: Compressor,
{
    pub async fn authorize_password(&mut self) -> ProcessResult<()> {
        match &self.config.authorization {
            AuthorizationBackend::Password { password } => {
                let supplied_password =
                    self.reader.read_string_prefixed().await?;
                if &supplied_password == password {
                    let new_rights =
                        self.config.rights.on_password_auth.to_bits();
                    self.user.rights = new_rights;
                    tracing::info!(
                        "{}'s rights updated: {new_rights:?}",
                        self.address.bold()
                    );
                    Ok(())
                } else {
                    Err(NonCriticalError::IncorrectUniversalPassword.into())
                }
            }
            AuthorizationBackend::Database { .. } => {
                let size = self.reader.read_u8().await? as usize;
                self.reader.skip_n_bytes::<64>(size).await?;
                Err(NonCriticalError::Unimplemented(
                    "authorize through database",
                )
                .into())
            }
        }
    }

    pub async fn ping(&mut self) -> ProcessResult<()> {
        self.writer
            .server()
            .write_ping(&PingResponseDescriptor {
                server_name: Cow::Borrowed(&self.config.server.name),
                buffer_read: self.config.server.buffering.read,
                compression: CompressionDescriptor {
                    algorithm: self.config.compression.algorithm,
                    level: self.config.compression.level,
                },
            })
            .await
            .map_err(Into::into)
    }
}
