use std::{
    fmt::Display,
    net::SocketAddr,
};

use owo_colors::OwoColorize;

#[derive(Debug)]
pub struct User {
    pub address: SocketAddr,
}

impl User {
    pub fn new(address: SocketAddr) -> Self {
        Self { address }
    }
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.address.bold())
    }
}
