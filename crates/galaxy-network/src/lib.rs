pub mod raw;

#[cfg(feature = "tokio")]
pub mod reader;

#[cfg(feature = "tokio")]
pub mod writer;

#[cfg(feature = "tokio")]
pub mod descriptors;

pub mod error;
mod utils;

#[cfg(test)]
mod tests;

pub use galaxy_shrinker as shrinker;
