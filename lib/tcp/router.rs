use std::io;

use mid_net::prelude::{
    impl_::interface::{
        ICompressor,
        IDecompressor,
    },
    *,
};

pub async fn run_tcp_packet_router<R, W, D, C>(
    mut reader: MidReader<R, D>,
    mut writer: MidWriter<W, C>,
) -> io::Result<()>
where
    R: ReaderUnderlyingExt,
    W: WriterUnderlyingExt,
    C: ICompressor,
    D: IDecompressor,
{
    loop {
        let (packet_type, packet_flags) =
            reader.read_raw_packet_type().await?;
    }
}
