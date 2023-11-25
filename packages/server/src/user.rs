use std::{
    fmt::Display,
    net::SocketAddr,
};

use flux_common::Rights;
use owo_colors::OwoColorize;

#[derive(Debug)]
pub struct User {
    pub rights: Rights,
    pub address: SocketAddr,
}

impl User {
    pub fn new(rights: Rights, address: SocketAddr) -> Self {
        Self { address, rights }
    }
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.address.bold())
    }
}
