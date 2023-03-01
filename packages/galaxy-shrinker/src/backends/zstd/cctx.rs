use std::{
    fmt::Debug,
    ptr::NonNull,
};

use zstd_sys::{
    ZSTD_CCtx,
    ZSTD_freeCCtx,
    ZSTD_isError,
};

pub struct ZStdCctx {
    ptr: NonNull<ZSTD_CCtx>,
    level: u8,
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
