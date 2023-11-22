use tokio::io::{
    AsyncReadExt,
    AsyncWriteExt,
};

pub trait RawRead: AsyncReadExt + Unpin {}
pub trait RawWrite: AsyncWriteExt + Unpin {}

impl<T: AsyncReadExt + Unpin> RawRead for T {}
impl<T: AsyncWriteExt + Unpin> RawWrite for T {}
