use std::{
    future::poll_fn,
    io,
    mem,
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
use tokio::io::ReadBuf;

use super::{
    destination::Destination,
    responses::NOT_FOUND,
    state::{
        Body,
        State,
    },
};
use crate::{
    config::HttpDiscoveryMethod,
    data::{
        commands::http::HttpMasterCommand,
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

// TODO: Implement discovery by path
pub struct Connection<R, W> {
    reader: R,
    pub writer: W,
    discovery: HttpDiscoveryMethod,

    buffer: Vec<u8>,
    forward_queue: ForwardQueue,

    cursor: usize,
    state: State,
    body: Body,
    pub dest: Option<Destination>,

    collection: Arc<EndpointCollection>,
}

impl<R, W> Connection<R, W>
where
    R: Read,
    W: Write,
{
    async fn read_chunks(&mut self) -> HttpResult<()> {
        let start_cursor = self.cursor;
        let able_to_overwrite = self.buffer.capacity() - start_cursor - 1;
        // < CRLF + single character
        // Client will not be able to write any byte data
        // to this buffer
        if able_to_overwrite < 3 {
            return Err(HttpError::BufferExhausted);
        }
        let mut overwritten = false;

        loop {
            let range = match self.read_line().await {
                Ok(r) => r,
                Err(HttpError::BufferExhausted) => {
                    if overwritten {
                        // if we previously overwritten
                        // the data - this is the signal that chunk
                        // size will not fit
                        // in the allocated buffer.
                        return Err(HttpError::BufferExhausted);
                    }
                    let current_cursor = self.cursor;
                    let copy_n = self.buffer.len() - current_cursor + 1;

                    unsafe {
                        ptr::copy(
                            self.buffer.as_ptr().add(current_cursor),
                            self.buffer.as_mut_ptr().add(start_cursor),
                            copy_n,
                        );
                        self.buffer.set_len(start_cursor + copy_n);
                    }

                    self.cursor = start_cursor;
                    overwritten = true;
                    continue;
                }

                Err(e) => return Err(e),
            };
            overwritten = false;
            let line = self.take_range(range);

            let chunk_size =
                parse_hex(line.strip_suffix(b"\r").unwrap_or(line))
                    .ok_or(HttpError::InvalidChunkSize)?;
            let (buffered, unbuffered) =
                self.calc_buffered((chunk_size + 2) as usize);

            if let Some(ref dst) = self.dest {
                let (buffered_crlf, recv_crlf) = match unbuffered {
                    n @ 0..=2 => (2 - n, n),
                    _ => (0, 2),
                };
                if buffered != 0 {
                    let slice = &self.buffer
                        [self.cursor..self.cursor + buffered - buffered_crlf];
                    dst.send(HttpMasterCommand::BodyChunk {
                        buf: slice.into(),
                    })?;
                }

                Self::read_and_send_chunked(
                    self.buffer.spare_capacity_mut(),
                    dst,
                    &mut self.reader,
                    chunk_size as usize,
                    recv_crlf,
                )
                .await?;
            } else {
                self.cursor += buffered;
            }

            if unbuffered != 0 {
                self.skip_data_of_size(unbuffered).await?;
            }

            if chunk_size == 0 {
                break;
            }
        }
        Ok(())
    }

    async fn handle_body(&mut self) -> HttpResult<()> {
        'handling: {
            match self.body {
                Body::ContentLength(length) => {
                    let cursor = self.cursor;
                    let (buffered, unbuffered) = self.calc_buffered(length);
                    self.cursor += buffered;

                    let Some(ref dest) = self.dest else {
                        self.skip_data_of_size(unbuffered).await?;
                        break 'handling;
                    };

                    if buffered != 0 {
                        dest.send(HttpMasterCommand::BodyChunk {
                            buf: Vec::from(
                                &self.buffer[cursor..cursor + buffered],
                            ),
                        })?;
                    }

                    if unbuffered != 0 {
                        Self::read_and_send_chunked(
                            self.buffer.spare_capacity_mut(),
                            dest,
                            &mut self.reader,
                            unbuffered,
                            0,
                        )
                        .await?;
                    }
                }

                Body::Chunked => {
                    self.read_chunks().await?;
                }
            }
        }

        if self.dest.is_none() {
            self.writer
                .write_all(NOT_FOUND.as_bytes())
                .await?;
        }

        self.reset_state();
        Ok(())
    }

    async fn read_and_send_chunked(
        buffer: &mut [mem::MaybeUninit<u8>],
        dst: &Destination,
        reader: &mut R,
        mut size: usize,
        crlf: usize,
    ) -> HttpResult<()> {
        let mut buffer = if buffer.len() < 512 {
            MaybeHeapChunk::heap(Vec::with_capacity(512))
        } else {
            MaybeHeapChunk::stack_uninit(buffer)
        };
        let buf_size = buffer.data().len();
        size += crlf;

        while size != 0 {
            let current_chunk = size.min(buf_size);
            let cur_read = Self::read_to_uninit(
                &mut *reader,
                &mut buffer.data_mut()[..current_chunk],
            )
            .await?;
            if cur_read == 0 {
                return Err(HttpError::Disconnected);
            }

            dst.send(HttpMasterCommand::BodyChunk {
                buf: Vec::from(unsafe { buffer.data_initialized(cur_read) }),
            })?;

            size -= cur_read;
        }

        Ok(())
    }

    async fn handle_header(&mut self, range: Range<usize>) -> HttpResult<()> {
        let line = self.take_range(range);
        if line == b"\r" {
            // Here is the body started
            return self.handle_body().await;
        }

        let Some((key, value)) =
            split_key_value(line) else {
                return Err(HttpError::MissingColon);
            };
        let value = value.strip_suffix(b"\r").unwrap_or(value);

        'check: {
            if case_insensitive_eq_left(key, b"HOST") {
                let host = strip_left_space(value);
                if host.is_empty() {
                    // Invalid host, so we just think that host is not present
                    self.dest = None;
                    break 'check;
                }

                match self.discovery {
                    HttpDiscoveryMethod::Host => {
                        let (id, permit) = {
                            let endpoints = self.collection.raw_endpoints();
                            let read_permit = endpoints.read().await;
                            let Some(endpoint) = read_permit.get(host) else {
                                break 'check;
                            };
                            endpoint.assign_id().await?
                        };
                        self.dest = Some(Destination {
                            id,
                            dest_id: host.into(),
                            permit,
                        });
                    }
                }
            } else if case_insensitive_eq_left(key, b"CONTENT-LENGTH") {
                self.body = Body::ContentLength(
                    parse_number(strip_left_space(value))
                        .ok_or(HttpError::InvalidContentLength)?
                        as _,
                );
            } else if case_insensitive_eq_left(key, b"TRANSFER-ENCODING")
                && is_chunked_transfer(value)
            {
                self.body = Body::Chunked;
            }
        }

        Ok(())
    }

    fn handle_request_line(&mut self, range: Range<usize>) {
        self.forward_queue.fill_request_line(range);
        self.state = State::Header;
    }
}

impl<R, W> Connection<R, W>
where
    R: Read,
    W: Write,
{
    pub async fn run(&mut self) -> HttpResult<()> {
        loop {
            let range = self.read_line().await?;
            match self.state {
                #[allow(clippy::unit_arg)]
                State::RequestLine => Ok(self.handle_request_line(range)),
                State::Header => self.handle_header(range).await,
            }?;
        }
    }

    pub fn new(
        reader: R,
        writer: W,
        allocate_buffer: usize,
        discovery: HttpDiscoveryMethod,
        collection: Arc<EndpointCollection>,
    ) -> Self {
        // hard mode Rust: we'll use only allocated buffer to store
        // data
        Self {
            buffer: Vec::with_capacity(allocate_buffer),
            discovery,

            reader,
            writer,
            cursor: 0,
            state: State::RequestLine,

            forward_queue: ForwardQueue::new(),
            body: Body::ContentLength(0),
            dest: None,
            collection,
        }
    }
}

impl<R, W> Connection<R, W>
where
    R: Read,
    W: Write,
{
    async fn read_line(&mut self) -> HttpResult<Range<usize>> {
        let mut continuation = 0;
        loop {
            let Some(newline_pos) = memchr(b'\n', &self.buffer[self.cursor + continuation..]) else {
                continuation += self.cursor_space();
                self.read_chunk().await?;
                continue;
            };
            let additional = newline_pos + continuation;
            let absolute_newline = self.cursor + additional;
            let return_range = self.cursor..absolute_newline;

            self.cursor += additional + 1/* newline */;

            break Ok(return_range);
        }
    }

    async fn read_chunk(&mut self) -> HttpResult<NonZeroUsize> {
        let unused_space = self.available_space();
        if unused_space == 0 {
            return Err(HttpError::BufferExhausted);
        }

        let read_no = {
            let spare = self.buffer.spare_capacity_mut();
            Self::read_to_uninit(&mut self.reader, spare)
                .await
                .map_err(Into::into)
                .and_then(|read_n| {
                    NonZeroUsize::new(read_n).ok_or(HttpError::Disconnected)
                })?
        };

        unsafe {
            self.buffer
                .set_len(self.buffer.len() + read_no.get());
        }

        Ok(read_no)
    }

    async fn skip_data_of_size(&mut self, mut size: usize) -> io::Result<()> {
        let free = self.available_space();
        let mut buffer = if free < 512 {
            MaybeHeapChunk::heap(Vec::with_capacity(512))
        } else {
            MaybeHeapChunk::stack_uninit(self.buffer.spare_capacity_mut())
        };
        let buffer_len = buffer.data().len();

        while size != 0 {
            let read_req = buffer_len.min(size);
            let cur_read = Self::read_to_uninit(
                &mut self.reader,
                &mut buffer.data_mut()[..read_req],
            )
            .await?;
            size -= cur_read;
        }

        Ok(())
    }

    async fn read_to_uninit<R0: Read>(
        mut reader: R0,
        dst: &mut [mem::MaybeUninit<u8>],
    ) -> io::Result<usize> {
        let mut read_buf = ReadBuf::uninit(dst);
        poll_fn(|cx| Pin::new(&mut reader).poll_read(cx, &mut read_buf))
            .await?;

        Ok(read_buf.filled().len())
    }

    fn reset_state(&mut self) {
        self.state = State::RequestLine;
        self.body = Body::ContentLength(0);
        self.forward_queue.reset();

        // TODO: Copy remaining contents to the start
        let quarter_cap = (self.buffer.capacity() >> 2).min(1024);
        if self.available_space() < quarter_cap {
            self.move_contents_to_start();
        }
    }

    fn calc_buffered(&self, size: usize) -> (usize, usize) {
        let buffered = self.cursor_space().min(size);
        (buffered, size - buffered)
    }

    fn move_contents_to_start(&mut self) {
        let data_start = self.cursor;
        let data_end = self.buffer.len();
        let data_len = data_end - data_start;

        let data_start_ptr = unsafe { self.buffer.as_ptr().add(data_start) };
        let start_ptr = self.buffer.as_mut_ptr();

        // Simple fast path, could be even faster but I don't want
        // to think about it for now
        if data_len > data_start {
            unsafe { ptr::copy(data_start_ptr, start_ptr, data_len) }
        } else {
            // SAFETY: safe since we're checked that regions do not
            // overlap
            unsafe {
                ptr::copy_nonoverlapping(data_start_ptr, start_ptr, data_len);
            }
        }

        self.cursor = 0;
        unsafe { self.buffer.set_len(data_len) };
    }

    fn take_range(&self, range: Range<usize>) -> &[u8] {
        &self.buffer[range]
    }

    fn cursor_space(&self) -> usize {
        self.buffer.len() - self.cursor
    }

    fn available_space(&self) -> usize {
        self.buffer.capacity() - self.buffer.len()
    }
}
