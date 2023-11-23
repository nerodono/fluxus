use std::{
    path::Path,
    process,
};

use color_eyre::eyre;
use fluxus::config::root::Config;

fn search_in_paths(paths: &[&Path]) -> Config {
    for path in paths {
        if let Ok(config) = Config::try_load(path) {
            return config;
        }
    }

    for path in paths {
        tracing::error!(" - {}", path.display());
    }

    process::exit(1)
}

pub fn load_config(path: Option<&Path>) -> eyre::Result<Config> {
    match path {
        Some(exact_path) => Config::try_load(exact_path),
        None => Ok(search_in_paths(&[
            Path::new("/etc/fluxus.toml"),
            Path::new("./fluxus.toml"),
        ])),
    }
}
