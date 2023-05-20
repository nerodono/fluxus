use std::{
    fmt::Display,
    num::{
        NonZeroU8,
        NonZeroUsize,
    },
    ptr::NonNull,
};

use zstd_sys::{
    ZSTD_CCtx,
    ZSTD_DCtx,
    ZSTD_compressCCtx,
    ZSTD_createCCtx,
    ZSTD_createDCtx,
    ZSTD_decompressDCtx,
    ZSTD_freeCCtx,
    ZSTD_freeDCtx,
    ZSTD_getDecompressedSize,
    ZSTD_isError,
};

use crate::traits::{
    CompressionError,
    Compressor,
    Decompressor,
};

pub struct ZStdCctx {
    cctx: NonNull<ZSTD_CCtx>,
    level: NonZeroU8,
}

#[repr(transparent)]
pub struct ZStdDctx {
    dctx: NonNull<ZSTD_DCtx>,
}

impl Decompressor for ZStdDctx {
    fn try_decompress(
        &mut self,
        src: &[u8],
        size_predicate: impl FnOnce(NonZeroUsize) -> bool,
    ) -> Option<Vec<u8>> {
        let size = unsafe {
            ZSTD_getDecompressedSize(src.as_ptr().cast(), src.len())
                .try_into()
                .ok()
                .and_then(NonZeroUsize::new)
        }?;
        if size_predicate(size) {
            let mut allocated: Vec<u8> = Vec::with_capacity(size.get());
            let length = unsafe {
                ZSTD_decompressDCtx(
                    self.dctx.as_ptr(),
                    allocated.as_mut_ptr().cast(),
                    allocated.capacity(),
                    src.as_ptr().cast(),
                    src.len(),
                )
            };
            if zstd_is_error(length) {
                None
            } else {
                unsafe { allocated.set_len(length) };
                Some(allocated)
            }
        } else {
            None
        }
    }
}

impl Compressor for ZStdCctx {
    fn try_compress_into(
        &mut self,
        src: &[u8],
        dst: &mut Vec<u8>,
    ) -> Result<(), CompressionError> {
        let result = unsafe {
            ZSTD_compressCCtx(
                self.cctx.as_ptr(),
                dst.as_mut_ptr().cast(),
                dst.capacity(),
                src.as_ptr().cast(),
                src.len(),
                self.level.get() as i32,
            )
        };

        if zstd_is_error(result) {
            Err(CompressionError)
        } else {
            unsafe { dst.set_len(result) };
            Ok(())
        }
    }
}

impl ZStdCctx {
    /// # Panics
    ///
    /// panics if underlying call to the [`ZSTD_createCCtx`]
    /// fails
    pub fn new(level: NonZeroU8) -> Self {
        let cctx = NonNull::new(unsafe { ZSTD_createCCtx() })
            .expect("failed to allocate zst compression context");
        Self { cctx, level }
    }
}

impl ZStdDctx {
    /// # Panics
    ///
    /// panics if underlying call to the [`ZSTD_createDCtx`]
    /// fails
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ZStdDctx {
    fn default() -> Self {
        let ptr = NonNull::new(unsafe { ZSTD_createDCtx() })
            .expect("failed to allocate zstd decompression context");
        Self { dctx: ptr }
    }
}

impl Drop for ZStdCctx {
    fn drop(&mut self) {
        panic_if_error(
            unsafe { ZSTD_freeCCtx(self.cctx.as_ptr()) },
            "failed to deallocate zstd compression context",
        );
    }
}

impl Drop for ZStdDctx {
    fn drop(&mut self) {
        panic_if_error(
            unsafe { ZSTD_freeDCtx(self.dctx.as_ptr()) },
            "failed to deallocate zstd decompression context",
        );
    }
}

fn panic_if_error(code: usize, msg: impl Display) {
    assert!(!zstd_is_error(code), "{msg}");
}

fn zstd_is_error(code: usize) -> bool {
    (unsafe { ZSTD_isError(code) }) == 1
}

unsafe impl Send for ZStdCctx {}
unsafe impl Send for ZStdDctx {}
