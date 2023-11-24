use integral_enum::integral_enum;
use thiserror::Error;

#[derive(Error)]
#[integral_enum]
pub enum NonCriticalError {
    #[error("failed to authenticate")]
    FailedToAuthenticate,
}

#[derive(Error)]
#[integral_enum]
pub enum CriticalError {
    #[error("unexpected packet for the client-side")]
    UnexpectedPacket,
}