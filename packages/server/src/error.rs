use integral_enum::integral_enum;
use thiserror::Error;

#[derive(Error)]
#[integral_enum]
pub enum NonCriticalError {
    #[error("failed to authenticate")]
    FailedToAuthenticate,

    #[error("access denied")]
    AccessDenied,
}

#[derive(Error)]
#[integral_enum]
pub enum CriticalError {
    #[error("unexpected packet for the client-side")]
    UnexpectedPacket,

    #[error("master channel closed (this is unexpected)")]
    ChannelClosed,

    #[error("the proxy was shut")]
    ServerWasShut,

    #[error("failed to bind address")]
    FailedToBind,
}
