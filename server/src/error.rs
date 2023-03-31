use integral_enum::integral_enum;
use thiserror::Error;

#[derive(Error)]
#[integral_enum]
pub enum CreateClientError {
    #[error("New ID would overflow underlying type")]
    IdWouldOverflow,
}

#[derive(Error)]
#[integral_enum]
pub enum ChanSendError {
    #[error("Requested ID does not exists")]
    IdDoesNotExists,

    #[error("Requested channel is closed")]
    ChannelIsClosed,
}
