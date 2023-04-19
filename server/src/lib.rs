#![warn(
    clippy::nursery,
    clippy::perf,
    clippy::correctness,
    clippy::pedantic,
    clippy::style,
    clippy::complexity
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::future_not_send
)]

mod decl;

pub mod error;
pub mod events;
pub mod protocols;
pub mod slaves;

pub mod config;
pub mod data;
pub mod utils;
