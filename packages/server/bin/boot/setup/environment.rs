use std::path::PathBuf;

use color_eyre::eyre;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Environment {
    pub config_path: Option<PathBuf>,
}

impl Environment {
    pub fn try_parse() -> eyre::Result<Self> {
        envy::prefixed("FLUXUS_")
            .from_env()
            .map_err(From::from)
    }
}
