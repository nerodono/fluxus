use integral_enum::integral_enum;
use thiserror::Error;

#[integral_enum]
#[derive(Error)]
pub enum PermitSendError {
    #[error("The channel is closed")]
    Closed,
}

#[integral_enum]
#[derive(Error)]
pub enum SendCommandError {
    #[error("The channel is closed")]
    Closed,

    #[error("Client not found")]
    ClientNotFound,
}
