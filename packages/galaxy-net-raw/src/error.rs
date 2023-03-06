use integral_enum::IntegralEnum;
use thiserror::Error;

#[derive(IntegralEnum, Error)]
#[repr(u8)]
pub enum Failure {
    #[error("Unknown command received")]
    UnknownCommand = 0,
}
