use tokio::io::AsyncWriteExt;

pub trait WriteExt: AsyncWriteExt + Unpin {}
impl<T: AsyncWriteExt + Unpin> WriteExt for T {}
