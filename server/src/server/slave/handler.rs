use std::net::SocketAddr;

use flux_compression::traits::Compressor;
use owo_colors::OwoColorize;
use tokio::{
    io::{
        AsyncReadExt,
        AsyncWriteExt,
    },
    net::TcpStream,
    sync::mpsc,
};

use crate::{
    communication::{
        flow::FlowCommand,
        slave::SlaveCommand,
    },
    config::CompressionScope,
    utils::buffer_ring::BufferRing,
};

const TRACE_TRIES: usize = 32;
const MIN_SUCC_PART: usize = 2; // 2^-2 (1/4)

pub async fn run_tcp_slave_handler<C>(
    allocate: usize,
    creator: SocketAddr,
    address: SocketAddr,
    mut stream: TcpStream,

    mut compression: Option<(C, CompressionScope)>,
    flow_tx: mpsc::Sender<FlowCommand>,
    mut slave_rx: mpsc::Receiver<SlaveCommand>,
) where
    C: Compressor + 'static,
{
    fn clone_if_not_reclaimed(
        reclaimed: Option<Vec<u8>>,
        read: usize,
        buffer: &[u8],
    ) -> Vec<u8> {
        reclaimed.map_or_else(
            || buffer[..read].to_owned(),
            |mut v| {
                // vector would only shrink
                unsafe { v.set_len(read) };
                v
            },
        )
    }

    let mut ring: BufferRing<64> = BufferRing::new();
    let mut buffer = vec![0; allocate];

    let mut compressed_num = 0_usize;
    let mut uncompressed_num = 0_usize;
    let mut tracing = true;

    loop {
        let read;
        let mut reclaimed = ring.try_pop();
        let read_to = if let Some(ref mut reclaimed_mut) = reclaimed {
            reclaimed_mut.as_mut_slice()
        } else {
            buffer.as_mut_slice()
        };

        tokio::select! {
            command = slave_rx.recv() => {
                let Some(command) = command else { break };
                match command {
                    SlaveCommand::Forward { buffer } => {
                        if stream.write_all(&buffer).await.is_err() {
                            break;
                        }

                        if buffer.len() >= 256 {
                            _ = ring.push(buffer);
                        }
                    }

                    SlaveCommand::Disconnect => {
                        break;
                    }
                }

                continue;
            }

            read_result = stream.read(read_to) => {
                let Ok(r @ 1..) = read_result else { break };
                read = r;
            }
        }

        let mut compressed = false;
        let mut turn_off_compression = false;
        let send_buffer = if let Some((cctx, settings)) = &mut compression {
            let total = compressed_num + uncompressed_num;
            if tracing && total >= TRACE_TRIES {
                let min_num = total.wrapping_shr(MIN_SUCC_PART as _);
                if compressed_num < min_num {
                    turn_off_compression = true;
                }

                tracing = false;
            }

            if read >= settings.threshold.get() as usize {
                let mut allocated = Vec::with_capacity(read);
                cctx.try_compress_into(&read_to[..read], &mut allocated)
                    .map_or_else(
                        |_| {
                            uncompressed_num =
                                uncompressed_num.wrapping_add(1);
                            clone_if_not_reclaimed(reclaimed, read, &buffer)
                        },
                        |()| {
                            compressed_num = compressed_num.wrapping_add(1);
                            compressed = true;
                            allocated
                        },
                    )
            } else {
                clone_if_not_reclaimed(reclaimed, read, &buffer)
            }
        } else {
            clone_if_not_reclaimed(reclaimed, read, &buffer)
        };

        if turn_off_compression {
            compression = None;
        }

        if flow_tx
            .send(FlowCommand::Forward {
                buffer: send_buffer,
                compressed,
            })
            .await
            .is_err()
        {
            break;
        }
    }

    tracing::info!(
        "{} disconnected from the {}'s server",
        address.bold(),
        creator.bold()
    );

    _ = flow_tx.send(FlowCommand::Disconnect).await;
}
