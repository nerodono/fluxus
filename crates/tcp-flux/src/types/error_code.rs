use integral_enum::integral_enum;
use thiserror::Error;

#[derive(Error)]
#[integral_enum(u8)]
pub enum ErrorCode {
    #[error("functionality is disabled during server compilation")]
    OptedOut = 0x00,

    #[error("failed to authenticate")]
    AuthenticationFailure = 0x01,

    #[error("no rights to access requested functionality (access denied)")]
    AccessDenied = 0x02,

    #[error("got unexpected packet")]
    UnexpectedPacket = 0x03,

    #[error("internal server error")]
    InternalError = 0x04,

    #[error("proxy server was shut")]
    Shutdown = 0x05,
}
