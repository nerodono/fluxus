use std::{
    num::{
        NonZeroU64,
        NonZeroUsize,
    },
    ptr::NonNull,
};

use negative_impl::negative_impl;
use zstd_sys::{
    ZSTD_DCtx,
    ZSTD_createDCtx,
    ZSTD_decompressDCtx,
    ZSTD_freeDCtx,
    ZSTD_getDecompressedSize,
    ZSTD_isError,
};

use crate::{
    error::DecompressError,
    interface::Decompressor,
};

pub struct ZStdDctx {
    ptr: NonNull<ZSTD_DCtx>,
}

impl Decompressor for ZStdDctx {
    fn try_get_size(&self, of: &[u8]) -> Option<NonZeroUsize> {
        let size = unsafe {
            ZSTD_getDecompressedSize(
                of.as_ptr() as *const _,
                of.len(),
            )
        };

        NonZeroU64::new(size)
            .map(TryInto::try_into)
            .and_then(Result::ok)
    }

    fn try_decompress<F>(
        &mut self,
        src: &[u8],
        buffer_allocator: F,
    ) -> Result<Vec<u8>, DecompressError>
    where
        F: FnOnce(Option<NonZeroUsize>) -> Option<Vec<u8>>,
    {
        let mut buffer = buffer_allocator(self.try_get_size(src))
            .ok_or(DecompressError::Declined)?;
        buffer.clear();

        let result = unsafe {
            ZSTD_decompressDCtx(
                self.ptr.as_ptr(),
                buffer.spare_capacity_mut().as_ptr() as *mut _,
                buffer.capacity(),
                src.as_ptr() as *const _,
                src.len(),
            )
        };

        if unsafe { ZSTD_isError(result) } == 1 {
            Err(DecompressError::TooShortBuffer)
        } else {
            unsafe { buffer.set_len(result) };
            Ok(buffer)
        }
    }
}

impl ZStdDctx {
    /// Creates the ZStandard decompression context.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ZStdDctx {
    fn default() -> Self {
        Self {
            ptr: NonNull::new(unsafe { ZSTD_createDCtx() }).expect(
                "Failed to allocate zstd decompression context",
            ),
        }
    }
}

impl Drop for ZStdDctx {
    fn drop(&mut self) {
        unsafe { ZSTD_freeDCtx(self.ptr.as_ptr()) };
    }
}

#[negative_impl]
impl !Sync for ZStdDctx {}
