use std::{
    num::NonZeroUsize,
    ops::RangeInclusive,
    ptr,
};

use zstd_sys::{
    ZSTD_CCtx,
    ZSTD_compressCCtx,
    ZSTD_createCCtx,
    ZSTD_freeCCtx,
    ZSTD_isError,
    ZSTD_maxCLevel,
};

use crate::{
    error,
    interface::ICompressor,
};

/// ZStandard compression context
pub struct ZStdCctx {
    ptr: ptr::NonNull<ZSTD_CCtx>,
    level: u8,
}

impl ICompressor for ZStdCctx {
    fn level(&self) -> usize {
        self.level as _
    }

    fn set_level(&mut self, level: usize) {
        self.level = level as _;
    }

    fn try_compress(
        &mut self,
        buf: &[u8],
        preallocated: &mut Vec<u8>,
    ) -> Result<std::num::NonZeroUsize, error::CompressError> {
        if !Self::levels_range().contains(&(self.level as usize)) {
            crate::cold();
            return Err(error::CompressError::InvalidLevel);
        }

        let result = unsafe {
            ZSTD_compressCCtx(
                self.ptr.as_ptr(),
                preallocated.as_ptr() as *mut _,
                preallocated.capacity(),
                buf.as_ptr() as *const _,
                buf.len() as _,
                self.level as _,
            )
        };

        if unsafe { ZSTD_isError(result) } == 0 {
            unsafe { preallocated.set_len(result) };
            Ok(unsafe { NonZeroUsize::new_unchecked(result) })
        } else {
            crate::cold();
            Err(error::CompressError::TooShortBuffer)
        }
    }

    fn supported_levels(&self) -> std::ops::RangeInclusive<usize> {
        Self::levels_range()
    }
}

impl ZStdCctx {
    /// Create ZStd compression context.
    ///
    /// # Panics
    ///
    /// - if underlying call to the `ZSTD_createCCtx`
    /// fails (returns NULL)
    /// - if level is invalid
    pub fn new(level: u8) -> Self {
        if !Self::levels_range().contains(&(level as usize)) {
            panic!(
                "Invalid level, supported levels are: {:?}",
                Self::levels_range()
            );
        }

        let cctx = unsafe { ZSTD_createCCtx() };
        let cctx = ptr::NonNull::new(cctx)
            .expect("Failed to create ZStdandard compression context");
        Self { ptr: cctx, level }
    }

    fn levels_range() -> RangeInclusive<usize> {
        1..=unsafe { ZSTD_maxCLevel() as usize }
    }
}

impl Drop for ZStdCctx {
    fn drop(&mut self) {
        if unsafe { ZSTD_isError(ZSTD_freeCCtx(self.ptr.as_ptr())) } == 1 {
            panic!("Failed to deallocate ZStd compression context");
        }
    }
}
