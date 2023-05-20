use std::{
    fs,
    net::SocketAddr,
    num::{
        NonZeroU16,
        NonZeroU8,
        NonZeroUsize,
    },
    path::Path,
};

use cfg_if::cfg_if;
use eyre::Context;
use flux_tcp::raw::CompressionAlgorithm;
use tracing::Level;
use url::Url;

use crate::{
    decl::config,
    traits::FromConfig,
};

const fn def_true() -> bool {
    true
}

// Main container
config! {
    struct Config {
        server: ServerConfig,
        buffering: BufferingConfig,

        compression: CompressionConfig,
        database: Option<DatabaseConfig>,

        rights: RightsConfig,
        runtime: RuntimeConfig,

        logging: LoggingConfig,
    }
}

// Logging
config! {
    int<u8> LogLevel { Disable, Debug, Error, Info }
    struct LoggingConfig {
        level: LogLevel,
    }
}

// Runtime

config! {
    struct RuntimeConfig {
        workers: NonZeroUsize,
    }
}

// Server
config! {
    struct ServerConfig {
        name: String,
        listen: SocketAddr,

        backlog: NonZeroUsize,
        max_uncompressed_size: NonZeroUsize,
        universal_password: Option<String>,
    }
}

// Buffering
config! {
    struct ReadBufferingScope {
        flow: NonZeroUsize,
        control: NonZeroUsize,
        per_slave: NonZeroUsize,
    }

    struct ChannelsBufferingScope {
        slave: NonZeroUsize,
        control: NonZeroUsize,
        flow: NonZeroUsize,
    }

    struct BufferingConfig {
        read: ReadBufferingScope,
        channels: ChannelsBufferingScope,
    }
}

// Compression
config! {
    #[derive(Clone)]
    struct CompressionScope {
        level: NonZeroU8,
        algorithm: CompressionAlgorithm,
        threshold: NonZeroU16,

        #[serde(default = "def_true")]
        trace: bool,
    }

    struct CompressionConfig {
        tcp: Option<CompressionScope>,
        http: Option<CompressionScope>,
    }
}

// Database

config! {
    struct DatabaseConfig {
        url: Url,
    }
}

// Rights

config! {
    struct HttpRightsScope {
        create: bool,
        custom_host: bool,
    }

    struct TcpRightsScope {
        create: bool,
        select_port: bool
    }

    struct RightsScope {
        tcp: TcpRightsScope,
        http: HttpRightsScope
    }

    struct RightsConfig {
        on_connect: RightsScope,
        on_universal_password: RightsScope,
    }
}

impl FromConfig<LogLevel> for Option<Level> {
    fn from_config(entry: &LogLevel) -> Self {
        Some(match *entry {
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Disable => return None,
            LogLevel::Error => Level::ERROR,
            LogLevel::Info => Level::INFO,
        })
    }
}

impl Config {
    pub fn load_config(path: impl AsRef<Path>) -> eyre::Result<Self> {
        cfg_if! {
            if #[cfg(feature = "dhall")] {
                let path = path.as_ref();
                let contents = fs::read_to_string(path)?;
                let is_dhall = path.extension().map_or(false, |ext| ext == "dhall");

                if is_dhall {
                    serde_dhall::from_str(&contents)
                               .parse()
                               .wrap_err("failed to parse dhall config")
                } else {
                    toml::from_str(&contents)
                        .wrap_err("failed to parse TOML")
                }
            } else {
                let contents = fs::read_to_string(path)
                                 .wrap_err("failed to read TOML config")?;

                toml::from_str(&contents)
                    .wrap_err("failed to parse TOML")
            }
        }
    }
}
