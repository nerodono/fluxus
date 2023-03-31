use std::{
    fs,
    io,
    net::SocketAddr,
    num::{
        NonZeroU8,
        NonZeroUsize,
    },
    path::Path,
};

use galaxy_network::raw::{
    CompressionAlgorithm,
    Rights,
};
use thiserror::Error;

use crate::decl::config;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to read config file: {0}")]
    Io(#[from] io::Error),

    #[error("Failed to parse TOML file: {0}")]
    Toml(#[from] toml::de::Error),
}

pub type ConfigResult<T> = Result<T, Error>;

config! {
    enum AuthorizationBackend {
        Password {
            password: String,
        },
        Database {
            url: String,
        },
    }

    int LogLevel<u8> { Disable, Info, Error, Debug }

    //

    struct CanConfig {
        create: bool,
        select_port: bool,
    }

    struct HttpRightsConfig {
        create: bool,
    }

    struct ProtocolRightsConfig {
        tcp: CanConfig,
        udp: CanConfig,

        http: HttpRightsConfig,
    }

    struct RightsConfig {
        on_connect: ProtocolRightsConfig,
        on_password_auth: ProtocolRightsConfig,
    }

    //

    struct CompressionConfig {
        algorithm: CompressionAlgorithm,
        level: NonZeroU8,
    }

    struct BufferingConfig {
        read: NonZeroUsize,
    }

    struct ServerConfig {
        listen: SocketAddr,
        name: String,
        buffering: BufferingConfig,
    }

    struct Config {
        server: ServerConfig,
        compression: CompressionConfig,
        authorization: AuthorizationBackend,
        rights: RightsConfig,
    }
}

impl ProtocolRightsConfig {
    pub fn to_bits(&self) -> Rights {
        let mut rights = Rights::empty();

        if self.tcp.create {
            rights |= Rights::CAN_CREATE_TCP;
        }
        if self.tcp.select_port {
            rights |= Rights::CAN_SELECT_TCP_PORT;
        }

        if self.udp.create {
            rights |= Rights::CAN_CREATE_UDP;
        }
        if self.udp.select_port {
            rights |= Rights::CAN_SELECT_UDP_PORT;
        }

        if self.http.create {
            rights |= Rights::CAN_CREATE_HTTP;
        }

        rights
    }
}

impl Config {
    /// Tries to load config from supplied paths.
    /// On success [`Config`] returned, otherwise array of
    /// errors returned.
    pub fn try_load_paths<P: AsRef<Path>>(
        paths: &[P],
    ) -> Result<Self, Vec<Error>> {
        let mut errors = Vec::new();
        for path in paths.into_iter().map(AsRef::as_ref) {
            match Self::try_load(path) {
                Ok(c) => return Ok(c),
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        Err(errors)
    }

    /// Tries to load config from supplied path.
    pub fn try_load(from: impl AsRef<Path>) -> ConfigResult<Self> {
        let content = fs::read_to_string(from)?;
        toml::from_str(&content).map_err(Into::into)
    }
}
