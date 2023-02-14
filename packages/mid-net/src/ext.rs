use tokio::{
    io::{
        AsyncReadExt,
        AsyncWriteExt,
        BufReader,
    },
    net::{
        tcp::{
            ReadHalf,
            WriteHalf,
        },
        TcpStream,
    },
};

use crate::{
    reader::MidReader,
    writer::MidWriter,
};

pub trait ReaderUnderlyingExt: AsyncReadExt + Unpin {}
pub trait WriterUnderlyingExt: AsyncWriteExt + Unpin {}

impl<T: AsyncReadExt + Unpin> ReaderUnderlyingExt for T {}
impl<T: AsyncWriteExt + Unpin> WriterUnderlyingExt for T {}

pub trait MidStreamExt {
    /// Create `Middleware` rw handles from the
    /// [`TcpStream`]
    fn as_rw_handles<C, D>(
        &mut self,
        compressor: C,
        decompressor: D,
    ) -> (MidReader<ReadHalf<'_>, D>, MidWriter<WriteHalf<'_>, C>);

    /// Create `Middleware` rw handles with the buffered
    /// reader
    fn as_buffered_rw_handles<C, D>(
        &mut self,
        compressor: C,
        decompressor: D,
        buffer_size: usize,
    ) -> (
        MidReader<BufReader<ReadHalf<'_>>, D>,
        MidWriter<WriteHalf<'_>, C>,
    );
}

impl MidStreamExt for TcpStream {
    fn as_rw_handles<C, D>(
        &mut self,
        compressor: C,
        decompressor: D,
    ) -> (MidReader<ReadHalf<'_>, D>, MidWriter<WriteHalf<'_>, C>) {
        let (read, write) = self.split();
        (
            MidReader::new(read, decompressor),
            MidWriter::new(write, compressor),
        )
    }

    fn as_buffered_rw_handles<C, D>(
        &mut self,
        compressor: C,
        decompressor: D,
        buffer_size: usize,
    ) -> (
        MidReader<BufReader<ReadHalf<'_>>, D>,
        MidWriter<WriteHalf<'_>, C>,
    ) {
        let (read, write) = self.split();
        (
            MidReader::new_buffered(read, decompressor, buffer_size),
            MidWriter::new(write, compressor),
        )
    }
}
