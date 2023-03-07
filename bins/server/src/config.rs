use std::{
    net::SocketAddr,
    num::NonZeroUsize,
};

pub use appconf::interface::ParserFunctionality;
use appconf::macros::decl;
use galaxy_net::raw::related::CompressionMethod;

#[decl]
pub struct PermissionEntry {
    pub can_create: bool,
    pub can_select_port: bool,
}

#[decl]
pub struct PermissionEntries {
    pub tcp: PermissionEntry,
}

#[decl]
pub struct PermissionStateEntries {
    pub just_connected: PermissionEntries,
    pub universal_password_permit: PermissionEntries,
}

#[decl]
pub struct BufferizationEntry {
    pub read: NonZeroUsize,
    pub per_client: NonZeroUsize,
}

#[decl]
pub struct CompressionMethodEntry {
    pub level: NonZeroUsize,
    pub threshold: NonZeroUsize,
}

#[decl]
pub struct CompressionEntry {
    pub zstd: CompressionMethodEntry,

    #[serde(rename = "use")]
    pub use_: CompressionMethod,
}

#[decl]
pub struct ServerEntry {
    pub name: String,
    pub listen: SocketAddr,
    pub universal_password: String,

    pub worker_threads: NonZeroUsize,
    pub bufferization: BufferizationEntry,
}

#[decl(loader = "toml")]
pub struct Config {
    pub server: ServerEntry,
    pub compression: CompressionEntry,
    pub permissions: PermissionStateEntries,
}
