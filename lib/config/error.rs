use std::{
    fmt::Display,
    io,
    path::Path,
};

use thiserror::Error;

#[derive(Debug)]
pub struct NotFoundDiagnose {
    pub paths: Vec<&'static Path>,
    pub errors: Vec<ConfigLoadError>,
}

#[derive(Debug, Error)]
pub enum ConfigLoadError {
    #[error("I/O Error: {0}")]
    Io(#[from] io::Error),

    #[error("{0}")]
    Toml(#[from] toml::de::Error),
}

impl From<Vec<(&'static Path, ConfigLoadError)>> for NotFoundDiagnose {
    fn from(value: Vec<(&'static Path, ConfigLoadError)>) -> Self {
        Self {
            paths: value.iter().map(|(p, ..)| *p).collect(),
            errors: value.into_iter().map(|(.., e)| e).collect(),
        }
    }
}

impl Display for NotFoundDiagnose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        assert_eq!(self.paths.len(), self.errors.len());

        f.write_str("Failed to parse config file, tried paths:\n")?;

        for (&path, error) in self.paths.iter().zip(self.errors.iter()) {
            let err_str = format!("{error}");
            f.write_fmt(format_args!(
                "- {display_path}:\n{error_str}\n\n",
                display_path = path.display(),
                error_str = err_str.trim()
            ))?;
        }

        Ok(())
    }
}
