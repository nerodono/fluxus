use std::{
    future::poll_fn,
    mem::MaybeUninit,
    num::NonZeroUsize,
    ops::Range,
    pin::Pin,
    ptr,
    sync::Arc,
};

use galaxy_network::{
    reader::Read,
    writer::Write,
};
use memchr::memchr;
use tokio::{
    io::ReadBuf,
    sync::mpsc,
};

use super::{
    buffer::{
        BodyBytes,
        RequestBuffer,
    },
    destination::Destination,
    responses::NOT_FOUND,
    state::Body,
};
use crate::{
    config::HttpDiscoveryMethod,
    data::{
        commands::http::{
            HttpMasterCommand,
            HttpSlaveCommand,
        },
        forward_queue::ForwardQueue,
        http::collection::EndpointCollection,
    },
    error::{
        HttpError,
        HttpResult,
    },
    utils::{
        maybe_heap_chunk::MaybeHeapChunk,
        parsing::{
            case_insensitive_eq_left,
            is_chunked_transfer,
            parse_hex,
            parse_number,
            split_key_value,
            strip_left_space,
        },
    },
};

pub struct Connection<R, W> {
    reader: R,
    pub writer: W,

    forward_queue: ForwardQueue,
    #[allow(dead_code)]
    discovery: HttpDiscoveryMethod,
    buffer: RequestBuffer,
    endpoints: Arc<EndpointCollection>,
    body: Body,

    channel_capacity: NonZeroUsize,
}

impl<R, W> Connection<R, W>
where
    R: Read,
    W: Write,
{
    async fn handle_body(
        &mut self,
        dest: &mut Option<Destination>,
    ) -> HttpResult<()> {
        match &self.body {
            Body::ContentLength(length) => {
                let BodyBytes {
                    buffered,
                    unbuffered,
                } = self.buffer.calc_recv_sizes(*length);

                if buffered != 0 {
                    Destination::send_if_valid(&*dest, || {
                        HttpMasterCommand::Forward {
                            buffer: Vec::from(
                                &self.buffer.cursor_slice()[..buffered],
                            ),
                        }
                    })
                    .await?;
                    self.buffer.cursor += buffered;
                }

                if unbuffered != 0 {
                    self.read_and_send_chunked(unbuffered, &*dest)
                        .await?;
                }
            }

            Body::Chunked => {
                let available = self.buffer.free_space();
                if available < 16 {
                    return Err(HttpError::DecidedToNotHandleChunked);
                }

                let start_cursor = self.buffer.cursor;
                let mut overwritten = false;
                loop {
                    let result = self.read_line(dest).await;
                    let size_range = match result {
                        Ok(r) => r,
                        Err(HttpError::BufferExhausted) => {
                            if overwritten {
                                // If previously we overwritten the buffer
                                // this is a signal that server can't handle
                                // this chunk
                                return Err(
                                    HttpError::BufferExhaustedDuringChunkRecv,
                                );
                            }

                            let current_cursor = self.buffer.cursor;
                            let able_to_reclaim =
                                self.buffer.len() - current_cursor + 1;

                            unsafe {
                                ptr::copy(
                                    self.buffer.as_ptr().add(current_cursor),
                                    self.buffer
                                        .as_mut_ptr()
                                        .add(start_cursor),
                                    able_to_reclaim,
                                );
                                self.buffer
                                    .buffer
                                    .set_len(start_cursor + able_to_reclaim);
                            }

                            self.buffer.cursor = start_cursor;

                            overwritten = true;
                            continue;
                        }

                        Err(e) => {
                            return Err(e);
                        }
                    };
                    overwritten = false;

                    let size_data = self.buffer.take_range(size_range);
                    let chunk_size = parse_hex(
                        size_data
                            .strip_suffix(b"\r\n")
                            .unwrap_or(size_data),
                    )
                    .ok_or(HttpError::InvalidChunkSize)?
                        as usize;

                    let is_end = chunk_size == 0;
                    let BodyBytes {
                        buffered,
                        unbuffered,
                    } = self.buffer.calc_recv_sizes(chunk_size + 2);

                    if buffered != 0 {
                        let range =
                            self.buffer.cursor..self.buffer.cursor + buffered;
                        Destination::send_if_valid(&*dest, || {
                            HttpMasterCommand::Forward {
                                buffer: Vec::from(
                                    self.buffer.take_range(range),
                                ),
                            }
                        })
                        .await?;
                        self.buffer.cursor += buffered;
                    }

                    if unbuffered != 0 {
                        self.read_and_send_chunked(unbuffered, &*dest)
                            .await?;
                    }

                    if is_end {
                        break;
                    }
                }
            }
        }

        if let Some(dest) = Destination::valid_for_send_mut(dest) {
            dest.discovered = false;
        } else {
            self.writer
                .write_all(NOT_FOUND.as_bytes())
                .await?;
        }

        self.prepare_new_request();
        Ok(())
    }

    async fn handle_headers(
        &mut self,
        dest: &mut Option<Destination>,
    ) -> HttpResult<()> {
        loop {
            let line_range = self.read_line(dest).await?;
            let line = self.buffer.take_range(line_range);
            self.forward_queue.append_header(line.len());

            Destination::send_if_valid(&*dest, || {
                HttpMasterCommand::Forward {
                    buffer: line.to_owned(),
                }
            })
            .await?;

            if line == b"\r\n" {
                // Here's the body started
                return self.handle_body(dest).await;
            }

            let line = line.strip_suffix(b"\r\n").unwrap_or(line);
            let (key, value) =
                split_key_value(line).ok_or(HttpError::MissingColon)?;
            let value = strip_left_space(value);
            if case_insensitive_eq_left(key, b"HOST") {
                let raw_endpoints = self.endpoints.raw_endpoints();
                let raw_endpoints = raw_endpoints.read().await;
                let Some(endpoint) = raw_endpoints.get(value) else {
                    Destination::set_discovered_opt(dest, false);
                    continue;
                };

                if let Some(ref mut dest) = dest {
                    if dest.same_endpoint(value) {
                        let buf = self
                            .buffer
                            .take_range(self.forward_queue.range());

                        dest.discovered = true;
                        dest.send(HttpMasterCommand::Forward {
                            buffer: Vec::from(buf),
                        })
                        .await?;
                        continue;
                    }
                }

                let (tx, rx) = mpsc::channel(self.channel_capacity.get());
                let (id, permit) = endpoint
                    .assign_id(
                        tx,
                        Vec::from(
                            self.buffer
                                .take_range(self.forward_queue.range()),
                        ),
                    )
                    .await?;
                *dest =
                    Some(Destination::new(id, value.to_owned(), permit, rx));
            } else if case_insensitive_eq_left(key, b"CONTENT-LENGTH") {
                let length = parse_number(value)
                    .ok_or(HttpError::InvalidContentLength)?
                    as usize;
                self.body = Body::ContentLength(length);
            } else if case_insensitive_eq_left(key, b"TRANSFER-ENCODING")
                && is_chunked_transfer(value)
            {
                self.body = Body::Chunked;
            }
        }
    }

    async fn handle_command(
        &mut self,
        command: HttpSlaveCommand,
        dest: &mut Option<Destination>,
    ) -> HttpResult<()> {
        match command {
            HttpSlaveCommand::Forward { buf } => {
                self.writer.write_all(&buf).await?;
            }

            HttpSlaveCommand::Disconnect => {
                if let Some(ref mut dest) = dest {
                    dest.discovered = false;
                    dest.dont_notify();
                }

                *dest = None;
                return Err(HttpError::ServerDisconnected);
            }
        }

        Ok(())
    }

    pub async fn run(&mut self) -> HttpResult<()> {
        let mut dest: Option<Destination> = None;
        loop {
            let request_line_range = self.read_line(&mut dest).await?;
            self.forward_queue
                .fill_request_line(request_line_range);
            self.handle_headers(&mut dest).await?;
        }
    }
}

// Helpers

impl<R, W> Connection<R, W>
where
    R: Read,
    W: Write,
{
    async fn read_line(
        &mut self,
        dest: &mut Option<Destination>,
    ) -> HttpResult<Range<usize>> {
        loop {
            let (slice, offset) = self.buffer.search_slice();
            let Some(newline_pos) = memchr(b'\n', slice) else {
                let slice_len = slice.len();
                let _read;
                loop {
                    tokio::select! {
                        res = self.read_chunk() => {
                            _read = res?;
                            break;
                        }

                        command = Destination::recv_command(dest) => {
                            let command = command.ok_or(HttpError::ChannelClosed)?;
                            self.handle_command(command, dest).await?;
                            continue;
                        }
                    }
                }

                self.buffer.continuation += slice_len;

                continue;
            };

            let prev_cursor = self.buffer.cursor;
            let absolute_newline = offset + newline_pos;
            let new_cursor = absolute_newline + 1;

            self.buffer.cursor = new_cursor;
            self.buffer.continuation = 0;

            break Ok(prev_cursor..new_cursor);
        }
    }

    async fn read_chunk(&mut self) -> HttpResult<NonZeroUsize> {
        if self.buffer.free_space() == 0 {
            return Err(HttpError::BufferExhausted);
        }

        let read = Self::read_to_uninit(
            &mut self.reader,
            self.buffer.spare_capacity_mut(),
        )
        .await?;

        unsafe { self.buffer.add_size(read.get()) };

        Ok(read)
    }

    async fn read_and_send_chunked(
        &mut self,
        mut size: usize,
        dest: &Option<Destination>,
    ) -> HttpResult<()> {
        let mut buffer = if self.buffer.free_space() < 512 {
            MaybeHeapChunk::heap(Vec::with_capacity(512))
        } else {
            MaybeHeapChunk::stack_uninit(self.buffer.spare_capacity_mut())
        };
        let buffer_len = buffer.data().len();

        while size != 0 {
            let cur_chunk_size = size.min(buffer_len);
            let cur_chunk = Self::read_to_uninit(
                &mut self.reader,
                &mut buffer.data_mut()[..cur_chunk_size],
            )
            .await?
            .get();

            Destination::send_if_valid(dest, || HttpMasterCommand::Forward {
                buffer: Vec::from(unsafe {
                    buffer.data_initialized(cur_chunk)
                }),
            })
            .await?;

            size -= cur_chunk;
        }

        Ok(())
    }

    async fn read_to_uninit<R0: Read>(
        mut reader: R0,
        to: &mut [MaybeUninit<u8>],
    ) -> HttpResult<NonZeroUsize> {
        let additional = {
            let mut read_buf = ReadBuf::uninit(to);
            poll_fn(|cx| Pin::new(&mut reader).poll_read(cx, &mut read_buf))
                .await?;

            read_buf.filled().len()
        };

        NonZeroUsize::new(additional).ok_or(HttpError::Disconnected)
    }

    fn prepare_new_request(&mut self) {
        self.body = Body::ContentLength(0);
        self.forward_queue.reset();
        self.buffer.move_contents();
    }
}

//

impl<R, W> Connection<R, W> {
    pub fn new(
        reader: R,
        writer: W,
        buffer_size: usize,
        discovery: HttpDiscoveryMethod,
        endpoints: Arc<EndpointCollection>,
        channel_capacity: NonZeroUsize,
    ) -> Self {
        Self {
            reader,
            writer,
            forward_queue: ForwardQueue::new(),
            discovery,
            buffer: RequestBuffer::new(buffer_size),
            endpoints,
            body: Body::ContentLength(0),
            channel_capacity,
        }
    }
}
