use std::io;

use galaxy_network::{
    shrinker::interface::Compressor,
    writer::{
        GalaxyWriter,
        Write,
    },
};

use crate::data::{
    command::erased::ErasedCommand,
    user::User,
};

#[inline]
pub async fn dispatch_command<W, C>(
    writer: &mut GalaxyWriter<W, C>,
    command: ErasedCommand,
    user: &mut User,
) -> io::Result<()>
where
    W: Write,
    C: Compressor,
{
    todo!()
}
