use std::num::NonZeroU16;

use clap::{
    Parser,
    Subcommand,
};

#[derive(Debug, Subcommand)]
pub enum CliSub {
    /// Create TCP proxy
    Tcp {
        /// Server of the local address
        local: String,

        /// Port to bind on the remote side
        #[clap(long, short)]
        port: Option<NonZeroU16>,
    },
}

#[derive(Debug, Parser)]
pub struct CliArgs {
    /// Remote address of the Neogrok server
    #[clap(long, short)]
    pub remote: String,

    /// Universal password to auth
    #[clap(long, short)]
    pub password: Option<String>,

    /// Action to perform
    #[clap(subcommand)]
    pub sub: CliSub,
}
