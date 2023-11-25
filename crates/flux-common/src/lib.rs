use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Rights: u16 {
        const CAN_PICK_TCP_PORT     = 1 << 0;
        const CAN_PICK_HTTP_DOMAIN  = 1 << 1;

        const CAN_CREATE_TCP_PROXY  = 1 << 2;
        const CAN_CREATE_HTTP_PROXY = 1 << 3;
    }
}
