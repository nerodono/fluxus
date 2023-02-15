use integral_enum::IntegralEnum;
use thiserror::Error;

#[derive(Error, IntegralEnum)]
pub enum IdRequestError {
    #[error("Ran out of identifiers")]
    Exceeded,
}
