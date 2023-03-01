use std::net::SocketAddr;

pub use appconf::interface::ParserFunctionality;
use appconf::macros::decl;

#[decl]
pub struct ServerEntry {
    pub listen: SocketAddr,
    pub universal_password: String,
}

#[decl(loader = "toml")]
pub struct Config {
    pub server: ServerEntry,
}
