use tokio::io::{
    AsyncReadExt,
    AsyncWriteExt,
};

pub trait ComposeRead: AsyncReadExt + Unpin {}
pub trait ComposeWrite: AsyncWriteExt + Unpin {}

impl<T: AsyncReadExt + Unpin> ComposeRead for T {}
impl<T: AsyncWriteExt + Unpin> ComposeWrite for T {}
