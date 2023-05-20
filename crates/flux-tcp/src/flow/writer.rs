use std::io;

use crate::{
    common::write_two_bufs,
    raw::FlowPacketFlags,
    traits::ComposeWrite,
};

pub struct FlowWriter<W> {
    writer: W,
}

impl<W: ComposeWrite> FlowWriter<W> {
    pub async fn write_forward(
        &mut self,
        flags: FlowPacketFlags,
        buffer: &[u8],
    ) -> io::Result<()> {
        let len = buffer.len() as u16;
        write_two_bufs(
            &mut self.writer,
            &[flags.bits(), (len & 0xff) as u8, (len >> 8) as u8],
            buffer,
        )
        .await
    }
}

impl<W> FlowWriter<W> {
    pub const fn new(writer: W) -> Self {
        Self { writer }
    }
}
