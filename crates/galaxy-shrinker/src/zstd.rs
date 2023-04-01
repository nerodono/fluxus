use std::{
    marker::PhantomData,
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

use crate::interface::{
    Compressor,
    Decompressor,
};

pub struct ZStdCctx {
    cctx: NonNull<ZSTD_CCtx>,
    _phantom: PhantomData<ZSTD_CCtx>,

    pub level: NonZeroU8,
}

pub struct ZStdDctx {
    dctx: NonNull<ZSTD_DCtx>,
    _phantom: PhantomData<ZSTD_DCtx>,
}

unsafe impl Send for ZStdCctx {}
impl !Sync for ZStdCctx {}

unsafe impl Send for ZStdDctx {}
impl !Sync for ZStdDctx {}

impl Decompressor for ZStdDctx {
    fn try_get_decompressed_size(
        &self,
        src: &[u8],
    ) -> Option<NonZeroUsize> {
        let size = unsafe {
            ZSTD_getDecompressedSize(src.as_ptr() as *const _, src.len())
        };
        if size != 0 {
            NonZeroUsize::new(size as _)
        } else {
            None
        }
    }

    fn try_decompress(
        &mut self,
        src: &[u8],
        preallocate: usize,
    ) -> Option<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::with_capacity(preallocate);
        unsafe {
            let spare = buffer.spare_capacity_mut();
            let size = ZSTD_decompressDCtx(
                self.dctx.as_ptr(),
                spare.as_ptr() as *mut _,
                spare.len(),
                src.as_ptr() as *const _,
                src.len(),
            );

            if ZSTD_isError(size) == 1 {
                None
            } else {
                buffer.set_len(size);
                Some(buffer)
            }
        }
    }
}

impl Compressor for ZStdCctx {
    fn try_compress(&mut self, src: &[u8]) -> Option<Vec<u8>> {
        let capacity = src.len();
        let mut buffer: Vec<u8> = Vec::with_capacity(capacity);
        unsafe {
            let spare = buffer.spare_capacity_mut();
            let result = ZSTD_compressCCtx(
                self.cctx.as_ptr(),
                spare.as_ptr() as *mut _,
                capacity,
                src.as_ptr() as *const _,
                src.len(),
                self.level.get() as _,
            );

            if ZSTD_isError(result) == 1 {
                None
            } else {
                buffer.set_len(result);
                Some(buffer)
            }
        }
    }
}

impl ZStdCctx {
    pub fn new(level: NonZeroU8) -> Self {
        let cctx = NonNull::new(unsafe { ZSTD_createCCtx() })
            .expect("Failed to create zstd compression ctx");
        Self {
            cctx,
            _phantom: PhantomData,
            level,
        }
    }
}

impl ZStdDctx {
    pub fn new() -> Self {
        let dctx = NonNull::new(unsafe { ZSTD_createDCtx() })
            .expect("Failed to create zstd decompression ctx");
        Self {
            dctx,
            _phantom: PhantomData,
        }
    }
}

impl Drop for ZStdDctx {
    fn drop(&mut self) {
        unsafe { ZSTD_freeDCtx(self.dctx.as_ptr()) };
    }
}

impl Drop for ZStdCctx {
    fn drop(&mut self) {
        unsafe { ZSTD_freeCCtx(self.cctx.as_ptr()) };
    }
}
