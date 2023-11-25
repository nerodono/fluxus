use std::io;

use crate::{
    connection::{
        master::payloads::info::InfoPayload,
        traits::RawWrite,
    },
    types::{
        error_code::ErrorCode,
        pkt_base::{
            PktBase,
            PktType,
        },
    },
};

pub struct MasterServerWriter<W> {
    writer: W,
}

impl<W: RawWrite> MasterServerWriter<W> {
    pub async fn write_error(&mut self, error: ErrorCode) -> io::Result<()> {
        self.writer
            .write_all(&[PktBase::simple(PktType::Error).encode(), error as u8])
            .await
    }

    pub async fn write_info(&mut self, info: InfoPayload<'_>) -> io::Result<()> {
        self.writer
            .write_all(&[
                PktBase::simple(PktType::ReqInfo).encode(),
                // TODO: include neccessary checks
                info.server_name.len() as u8,
            ])
            .await?;
        self.writer
            .write_all(info.server_name.as_bytes())
            .await
    }
}

impl<W> MasterServerWriter<W> {
    pub const fn new(writer: W) -> Self {
        Self { writer }
    }
}
