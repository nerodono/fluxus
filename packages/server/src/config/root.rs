use std::{
    fs,
    path::Path,
};

use color_eyre::eyre;

use super::{
    logging::LoggingConfig,
    protocols::ProtocolsConfig,
    runtime::RuntimeConfig,
};

entity! {
    struct Config {
        protocols: ProtocolsConfig,
        logging: LoggingConfig,

        #[serde(default)]
        runtime: RuntimeConfig,
    }
}

impl Config {
    pub fn try_load(path: impl AsRef<Path>) -> eyre::Result<Self> {
        let path = path.as_ref();
        tracing::debug!("Loading config from the {}", path.display());

        let contents = fs::read_to_string(path)?;
        toml::from_str(&contents).map_err(From::from)
    }
}
