use integral_enum::integral_enum;
use thiserror::Error;

#[integral_enum]
#[derive(Error)]
pub enum CommandSendError {
    #[error("No such client")]
    NoSuchClient,

    #[error("Channel is closed")]
    Closed,
}
