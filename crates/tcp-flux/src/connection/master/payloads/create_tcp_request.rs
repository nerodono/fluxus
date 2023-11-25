use std::num::NonZeroU16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreateTcpRequest {
    pub specific_port: Option<NonZeroU16>,
}
