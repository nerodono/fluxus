use std::io;

use galaxy_network::{
    reader::Read,
    shrinker::interface::{
        Compressor,
        Decompressor,
    },
    writer::Write,
};

use super::connection::Connection;

pub async fn handle_command<'a, R, D, W, C>(
    connection: &mut Connection<'a, R, D, W, C>,
) -> io::Result<()>
where
    R: Read,
    W: Write,
    D: Decompressor,
    C: Compressor,
{
    todo!()
}
