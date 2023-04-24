#![feature(type_alias_impl_trait)]
#![deny(
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
    clippy::future_not_send,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::unused_async
)]

mod decl;

pub mod error;
pub mod events;
pub mod protocols;
pub mod slaves;

pub mod features;

pub mod config;
pub mod data;
pub mod utils;
