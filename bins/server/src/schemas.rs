use galaxy_net::schemas::Permissions;

use crate::config::PermissionEntries;

impl From<&PermissionEntries> for Permissions {
    fn from(value: &PermissionEntries) -> Self {
        let mut flags = Permissions::empty();

        if value.tcp.can_create {
            flags |= Permissions::CAN_CREATE_TCP;
        }
        if value.tcp.can_select_port {
            flags |= Permissions::CAN_SELECT_TCP;
        }

        flags
    }
}
