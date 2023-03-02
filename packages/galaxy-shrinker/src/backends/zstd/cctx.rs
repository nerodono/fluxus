use std::{
    fmt::Debug,
    num::NonZeroU8,
    ptr::NonNull,
};

use negative_impl::negative_impl;
use zstd_sys::{
    ZSTD_CCtx,
    ZSTD_compressCCtx,
    ZSTD_createCCtx,
    ZSTD_freeCCtx,
    ZSTD_isError,
};

use crate::{
    error::CompressError,
    interface::Compressor,
};

pub struct ZStdCctx {
    ptr: NonNull<ZSTD_CCtx>,
    level: NonZeroU8,
}

impl Clone for ZStdCctx {
    fn clone(&self) -> Self {
        Self::new(self.level)
    }
}

impl Compressor for ZStdCctx {
    fn try_compress(
        &mut self,
        src: &[u8],
        dst: &mut Vec<u8>,
    ) -> Result<usize, CompressError> {
        dst.clear();
        let result = unsafe {
            ZSTD_compressCCtx(
                self.ptr.as_ptr(),
                dst.spare_capacity_mut().as_ptr() as *mut _,
                dst.capacity(),
                src.as_ptr() as *const _,
                src.len(),
                self.level.get() as i32,
            )
        };

        if unsafe { ZSTD_isError(result) } == 1 {
            Err(CompressError::InsufficientBuffer)
        } else {
            unsafe { dst.set_len(result) };
            Ok(result)
        }
    }
}

impl ZStdCctx {
    /// Allocates the compression context.
    ///
    /// # Panics
    ///
    /// Panics if underlying call to the [`ZSTD_createCCtx`]
    /// returned null
    pub fn new(level: NonZeroU8) -> Self {
        let cctx = unsafe { ZSTD_createCCtx() };
        Self {
            ptr: NonNull::new(cctx).expect(
                "Failed to allocate zstd compression context",
            ),
            level,
        }
    }
}

impl Debug for ZStdCctx {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("ZStdCctx")
            .field("level", &self.level)
            .finish()
    }
}

impl Drop for ZStdCctx {
    fn drop(&mut self) {
        unsafe { ZSTD_freeCCtx(self.ptr.as_ptr()) };
    }
}

unsafe impl Send for ZStdCctx {}

#[negative_impl]
impl !Sync for ZStdCctx {}
