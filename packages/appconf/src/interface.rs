use std::{
    fs,
    path::Path,
};

use crate::error::GenericError;

pub trait ParserFunctionality {
    type Error;

    /// Tries to parse specified string into the target
    /// format
    fn try_parse(text: &str) -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// Serializes your struct into the string.
    ///
    /// # Panic
    ///
    /// Panics if struct can't be serialized (serde format
    /// implementation error)
    fn serialize(&self, pretty: bool) -> String;

    /// Tries to load and parse specified file.
    fn try_load(
        path: impl AsRef<Path>,
    ) -> Result<Self, GenericError<Self::Error>>
    where
        Self: Sized,
    {
        let contents = fs::read_to_string(path)?;
        Self::try_parse(&contents).map_err(GenericError::Parser)
    }
}
