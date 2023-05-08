use std::num::{
    NonZeroU16,
    NonZeroUsize,
};

use clap::{
    Parser,
    Subcommand,
};

#[derive(Debug, Subcommand)]
pub enum CliSub {
    /// Create HTTP proxy
    Http {
        /// Local server address
        local: String,

        /// Domain to bind (Host header)
        #[clap(long, short)]
        domain: String,
    },

    /// Create TCP proxy server on the remote
    Tcp {
        /// Local server address
        local: String,

        /// Remote port to bind
        #[clap(long, short)]
        port: Option<NonZeroU16>,
    },
}

#[derive(Debug, Parser)]
pub struct CliArgs {
    /// Number of threads to use. Defaults to the number of
    /// logical CPUs
    #[clap(long, short)]
    pub workers: Option<NonZeroUsize>,

    /// Address of the remote fluxus server
    #[clap(long, short)]
    pub remote: String,

    /// Universal password for authorization
    #[clap(long, short)]
    pub password: Option<String>,

    #[clap(subcommand)]
    pub sub: CliSub,
}

impl CliArgs {
    pub fn parse() -> Self {
        Parser::parse()
    }
}
