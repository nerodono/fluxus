use std::num::NonZeroUsize;

use crate::error;

/// Generic trait for the compression routines
pub trait Compressor {
    /// Tries to compress supplied buffer to the `dst`
    /// buffer returning compressed size.
    fn try_compress(
        &mut self,
        src: &[u8],
        dst: &mut Vec<u8>,
    ) -> Result<usize, error::CompressError>;
}

/// Generic trait for the decompression routines
pub trait Decompressor {
    /// Tries to read compressed object size. Returns `None`
    /// if size could not be determined
    fn try_get_size(&self, of: &[u8]) -> Option<NonZeroUsize>;

    /// Tries to decompress data with custom logic in
    /// allocation.
    fn try_decompress<F>(
        &mut self,
        src: &[u8],
        buffer_allocator: F,
    ) -> Result<Vec<u8>, error::DecompressError>
    where
        F: FnOnce(Option<NonZeroUsize>) -> Option<Vec<u8>>;
}
