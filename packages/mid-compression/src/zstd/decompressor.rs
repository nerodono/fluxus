use std::ptr;

use zstd_sys::{
    ZSTD_DCtx,
    ZSTD_createDCtx,
    ZSTD_decompressDCtx,
    ZSTD_freeDCtx,
    ZSTD_getDecompressedSize,
    ZSTD_isError,
};

use crate::{
    error,
    interface::IDecompressor,
};

/// ZStandard decompression context
pub struct ZStdDctx {
    ptr: ptr::NonNull<ZSTD_DCtx>,
}

// TODO: Implement precise error handling
impl IDecompressor for ZStdDctx {
    fn try_decompress(
        &mut self,
        buffer: &[u8],
        to: &mut Vec<u8>,
    ) -> Result<usize, error::DecompressError> {
        let result = unsafe {
            ZSTD_decompressDCtx(
                self.ptr.as_ptr(),
                to.as_ptr() as *mut _,
                to.capacity(),
                buffer.as_ptr() as *const _,
                buffer.len(),
            )
        };

        if unsafe { ZSTD_isError(result) } == 1 {
            crate::cold();
            Err(error::DecompressError::InsufficientBuffer)
        } else {
            unsafe { to.set_len(result) };
            Ok(result)
        }
    }

    fn try_decompressed_size(
        &self,
        of: &[u8],
    ) -> Result<usize, error::SizeRetrievalError> {
        let Ok(result) = unsafe {
            ZSTD_getDecompressedSize(of.as_ptr() as *const _, of.len())
        }.try_into() else {
            return Err(error::SizeRetrievalError::InvalidData);
        };

        if unsafe { ZSTD_isError(result) } == 1 {
            // TODO: see trait todo
            Err(error::SizeRetrievalError::InvalidData)
        } else {
            Ok(result)
        }
    }
}

impl ZStdDctx {
    /// Create ZStandard decompression context.
    ///
    /// # Panics
    ///
    /// - if underlying call to the `ZSTD_createDCtx()`
    ///   fails (returns NULL pointer)
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ZStdDctx {
    fn default() -> Self {
        Self {
            ptr: ptr::NonNull::new(unsafe { ZSTD_createDCtx() })
                .expect("Failed to create ZStd decompression context"),
        }
    }
}

unsafe impl Send for ZStdDctx {}

impl Drop for ZStdDctx {
    fn drop(&mut self) {
        if unsafe { ZSTD_isError(ZSTD_freeDCtx(self.ptr.as_ptr())) } == 1 {
            panic!("Failed to deallocate ZStd decompression context");
        }
    }
}
