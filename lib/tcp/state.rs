use bitflags::bitflags;

use crate::config::base::ProtocolPermissionsCfg;

bitflags! {
    pub struct Permissions: u16 {
        const CREATE_TCP      = 1 << 0;
        const SELECT_TCP_PORT = 1 << 1;
    }
}

/// User state during the connection.
pub struct State {
    pub permissions: Permissions,
}

impl Permissions {
    pub fn from_cfg(permissions: &ProtocolPermissionsCfg) -> Self {
        let mut perms = Permissions::empty();

        if permissions.tcp.create_server {
            perms |= Permissions::CREATE_TCP;
        }
        if permissions.tcp.select_port {
            perms |= Permissions::SELECT_TCP_PORT;
        }

        perms
    }
}

impl State {
    /// Create state from the permissions configuration
    pub fn new(permissions: &ProtocolPermissionsCfg) -> Self {
        Self {
            permissions: Permissions::from_cfg(permissions),
        }
    }
}
