use std::{
    num::NonZeroUsize,
    path::Path,
};

use lazy_static::lazy_static;
use mid_net::compression::CompressionAlgorithm;

use super::error::NotFoundDiagnose;

fn default_threshold() -> NonZeroUsize {
    NonZeroUsize::new(64).unwrap()
}

super::config_entry! {
    struct ServerBufferizationCfg {
        /// Buffer for the read side of the socket
        read: usize,
    }

    struct ServerCfg {
        /// Address to bind to. For example (in `address:port` format):
        /// - `0.0.0.0:6567`
        /// - `127.0.0.1:6567`
        listen: String,

        /// Number of threads used by the server. If not specified,
        /// then value of `nproc` will be used(number of CPU logical processors)
        threads: Option<usize>,

        /// Server bufferization settings
        bufferization: ServerBufferizationCfg,
    }

    struct ProtocolCompressionCfg {
        /// Algorithm to use (zstd for example, case sensitive)
        algorithm: CompressionAlgorithm,

        /// Compression level, varies from algorithm to algorithm
        level: NonZeroUsize,

        /// Compression threshold in bytes.
        #[serde(default = "default_threshold")]
        threshold: NonZeroUsize,
    }

    struct CompressionCfg {
        /// Compression of the TCP relay
        tcp: ProtocolCompressionCfg,
    }

    struct Config {
        /// Server configuration
        server: ServerCfg,

        /// Compression configuration
        compression: CompressionCfg,
    }
}

impl Config {
    /// Get singleton immutable instance of the config
    pub fn instance() -> &'static Config {
        lazy_static! {
            static ref CFG: Config = Config::try_load_paths()
                .unwrap_or_else(|diagnose| {
                    eprintln!("{diagnose}");
                    panic!()
                })
                .0;
        }

        &CFG
    }

    /// Tries to load from bunch of paths, returns:
    /// - `Ok((Self, found_path))` if success
    /// - Diagnose struct, can be directly displayed
    pub fn try_load_paths(
    ) -> Result<(Self, &'static Path), NotFoundDiagnose> {
        let paths = [
            Path::new("config/config.toml"),
            Path::new("/etc/neogrok/config.toml"),
            Path::new("neogrok/config.toml"),
        ];
        let mut pairs = Vec::new();

        for path in paths {
            match Self::try_load(path) {
                Ok(self_) => {
                    return Ok((self_, path));
                }
                Err(e) => {
                    pairs.push((path, e));
                }
            }
        }

        Err(NotFoundDiagnose::from(pairs))
    }
}
