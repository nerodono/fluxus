use std::{
    any::type_name,
    fs,
    path::Path,
};

use super::error::ConfigLoadError;

/// Generic save function. Generally implemented as a
/// sequence of [`toml::to_string_pretty`] and
/// [`std::fs::write`] calls.
#[track_caller]
pub fn generic_save<Ser: serde::ser::Serialize>(
    ref_: &Ser,
    path: impl AsRef<Path>,
) {
    fs::write(
        path,
        toml::to_string_pretty(ref_).unwrap_or_else(|e| {
            panic!(
                "Failed to serialize config entry ({}): {e}",
                type_name::<Ser>()
            )
        }),
    )
    .unwrap_or_else(|e| {
        panic!(
            "Failed to write config entry ({}): {e}",
            type_name::<Ser>()
        )
    });
}

/// Generic load from path function. Used mainly to decrease
/// monomorphisation impact
pub fn try_generic_load<De: serde::de::DeserializeOwned>(
    path: impl AsRef<Path>,
) -> Result<De, ConfigLoadError> {
    let contents = fs::read_to_string(path)?;
    toml::from_str(&contents).map_err(|e| e.into())
}
