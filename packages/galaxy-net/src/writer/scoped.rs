use std::{
    future::Future,
    io::{
        self,
        IoSlice,
    },
    ptr,
};

use tokio::io::AsyncWriteExt;

use super::GalaxyWriter;
use crate::__raw_impl;

pub trait WriteExt: AsyncWriteExt + Unpin {}
impl<T: AsyncWriteExt + Unpin> WriteExt for T {}

pub struct RawWriter<W, C> {
    pub(crate) stream: GalaxyWriter<W, C>,
}

impl<W: WriteExt, C> RawWriter<W, C> {
    /// Writes two buffers in at least 1 syscall (fast
    /// path, likely to work).
    ///
    /// Note: if target writer does not support vectored
    /// writes, then intermediate buffer allocated
    /// and data will be copied inside it.
    pub async fn write_two_bufs(
        &mut self,
        prepend: &[u8],
        append: &[u8],
    ) -> io::Result<bool> {
        // :D
        fn true_((): ()) -> bool {
            true
        }

        let plen = prepend.len();
        let alen = append.len();
        let total = plen + alen;

        if !self.stream().is_write_vectored() {
            let mut buf: Vec<u8> = Vec::with_capacity(total);

            // Q: Possibly it could be done without any unsafe and
            // buffer pre-filling?
            unsafe {
                let spare = buf.spare_capacity_mut();
                let spare_ptr = spare.as_mut_ptr();
                ptr::copy_nonoverlapping(
                    prepend.as_ptr(),
                    spare_ptr as *mut _,
                    plen,
                );
                ptr::copy_nonoverlapping(
                    append.as_ptr(),
                    spare_ptr.add(plen) as *mut _,
                    alen,
                );

                buf.set_len(total);
            }

            return self
                .stream_mut()
                .write_all(&buf)
                .await
                .map(true_);
        }

        let slices = [IoSlice::new(prepend), IoSlice::new(append)];
        let mut written = 0;

        while written < total {
            let wrote =
                self.stream_mut().write_vectored(&slices).await?;
            written += wrote;
            if wrote >= plen {
                return self
                    .stream_mut()
                    .write_all(&append[(total - written)..])
                    .await
                    .map(true_);
            }
        }

        Ok(true)
    }

    /// Write supplied buffer into the stream.
    pub fn write_buffer<'a>(
        &'a mut self,
        buffer: &'a [u8],
    ) -> impl Future<Output = io::Result<()>> + 'a {
        self.stream_mut().write_all(buffer)
    }
}

impl<W, C> RawWriter<W, C> {
    __raw_impl! { @stream<W> stream.stream }

    __raw_impl! { @compressor<C> stream.compressor }
}
