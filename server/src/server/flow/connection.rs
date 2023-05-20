use std::{
    num::NonZeroUsize,
    ptr,
};

use flux_compression::{
    polymorphic::PolyDctx,
    traits::Decompressor,
};
use flux_tcp::{
    error::FlowReadResult,
    flow::{
        reader::FlowReader,
        writer::FlowWriter,
    },
    raw::FlowPacketFlags,
    traits::{
        ComposeRead,
        ComposeWrite,
    },
};
use tokio::{
    io::ReadBuf,
    sync::mpsc,
};

use crate::{
    communication::{
        flow::FlowCommand,
        slave::SlaveCommand,
    },
    data::server::QueuedConnection,
    error::{
        FlowProcessError,
        FlowProcessResult,
    },
    utils::maybe_deallocate_chunk::MaybeDeallocateChunk,
};

const MIN_SPARE_TO_STORE: usize = 512;

pub struct Connection<R, W> {
    reader: FlowReader<R>,
    writer: FlowWriter<W>,

    slave_tx: mpsc::Sender<SlaveCommand>,
    flow_rx: mpsc::Receiver<FlowCommand>,

    max_compressed_size: NonZeroUsize,
    max_uncompressed_size: NonZeroUsize,
    dctx: PolyDctx,
}

impl<R, W> Connection<R, W>
where
    R: ComposeRead,
    W: ComposeWrite,
{
    async fn handle_command(
        &mut self,
        command: FlowCommand,
    ) -> FlowReadResult<bool> {
        match command {
            FlowCommand::Forward { buffer, compressed } => {
                self.writer
                    .write_forward(
                        if compressed {
                            FlowPacketFlags::COMPRESSED
                        } else {
                            FlowPacketFlags::empty()
                        },
                        &buffer,
                    )
                    .await?;
            }
            FlowCommand::Disconnect => return Ok(true),
        }
        Ok(false)
    }

    async fn read_and_forward_chunked(
        &mut self,
        length: usize,
    ) -> FlowProcessResult<()> {
        let buffered = self.reader.consume_max(length);
        let mut unbuffered_len = length - buffered.len();
        if !buffered.is_empty() {
            self.slave_tx
                .send(SlaveCommand::Forward {
                    buffer: Vec::from(buffered),
                })
                .await?;
        }

        if unbuffered_len == 0 {
            return Ok(());
        }

        let (spare, raw_reader) = self.reader.spare_and_raw();
        let mut chunk =
            MaybeDeallocateChunk::new_sufficiency(spare, MIN_SPARE_TO_STORE);

        while unbuffered_len != 0 {
            let cur_read = length.min(chunk.len());
            let actually_read = {
                let mut read_buf =
                    ReadBuf::uninit(&mut chunk.data()[..cur_read]);

                let actually_read =
                    raw_reader.read_buf(&mut read_buf).await?;
                if actually_read == 0 {
                    return Err(FlowProcessError::DisconnectedDuringForward);
                }

                actually_read
            };

            let vec =
                Vec::from(unsafe { chunk.data_initialized(actually_read) });
            self.slave_tx
                .send(SlaveCommand::Forward { buffer: vec })
                .await?;

            unbuffered_len -= actually_read;
        }

        Ok(())
    }

    pub async fn serve(&mut self) -> FlowProcessResult<()> {
        loop {
            let flags;
            let length;

            tokio::select! {
                command = self.flow_rx.recv() => {
                    let Some(command) = command else {
                        break;
                    };
                    self.handle_command(command).await?;

                    continue;
                }

                read = self.reader.read_forward_header() => {
                    (flags, length) = read?;
                }
            }

            let length = length as usize;
            if flags.contains(FlowPacketFlags::COMPRESSED) {
                if length >= self.max_compressed_size.get() {
                    break;
                }

                let (reused, initialized, raw_reader) =
                    self.reader.take_slice_and_raw(length);
                let mut read_to = reused.map_or_else(
                    |init_buf| {
                        debug_assert_eq!(initialized, init_buf.len());
                        let mut vec = Vec::with_capacity(length);
                        unsafe {
                            ptr::copy_nonoverlapping(
                                init_buf.as_ptr(),
                                vec.as_mut_ptr(),
                                init_buf.len(),
                            );
                        }

                        MaybeDeallocateChunk::new_deallocated(vec, length)
                    },
                    |s| {
                        MaybeDeallocateChunk::new_sufficiency_clamped(
                            s, length,
                        )
                    },
                );

                {
                    let mut read_buf = ReadBuf::uninit(read_to.data());
                    let mut need_to_read = length - initialized;
                    while need_to_read != 0 {
                        let actually_read =
                            raw_reader.read_buf(&mut read_buf).await?;
                        if actually_read == 0 {
                            return Err(
                                FlowProcessError::DisconnectedDuringForward,
                            );
                        }

                        need_to_read -= actually_read;
                    }
                }

                let decompressed = self
                    .dctx
                    .try_decompress(
                        unsafe { read_to.full_data_initialized() },
                        |size| size <= self.max_uncompressed_size,
                    )
                    .ok_or(FlowProcessError::FailedToDecompress)?;
                self.slave_tx
                    .send(SlaveCommand::Forward {
                        buffer: decompressed,
                    })
                    .await?;
            } else {
                self.read_and_forward_chunked(length).await?;
            }
        }
        Ok(())
    }
}

impl<R, W> Connection<R, W>
where
    R: ComposeRead,
{
    pub fn new(
        dctx: PolyDctx,

        max_compressed_size: NonZeroUsize,
        max_uncompressed_size: NonZeroUsize,

        capacity: usize,
        reader: R,
        writer: W,
        queued: QueuedConnection,
    ) -> Self {
        Self {
            reader: FlowReader::with_capacity(reader, capacity),
            writer: FlowWriter::new(writer),

            slave_tx: queued.slave_tx,
            flow_rx: queued.flow_rx,

            max_compressed_size,
            max_uncompressed_size,

            dctx,
        }
    }
}
