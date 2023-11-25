use std::num::NonZeroU16;

#[derive(Debug)]
pub enum MasterEvent {
    BoundTcpPort { port: Option<NonZeroU16> },
}
