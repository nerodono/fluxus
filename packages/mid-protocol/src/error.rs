use integral_enum::IntegralEnum;
use thiserror::Error;

#[derive(IntegralEnum, Error)]
pub enum MidError {
    #[error("Unknown packet type")]
    UnknownPacket,
}
