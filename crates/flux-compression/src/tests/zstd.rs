use std::num::NonZeroU8;

use rstest::rstest;

use crate::{
    traits::{
        Compressor,
        Decompressor,
    },
    zstd::{
        ZStdCctx,
        ZStdDctx,
    },
};

const LEVEL: NonZeroU8 = unsafe { NonZeroU8::new_unchecked(12) };

#[rstest]
#[case(b"hello world")]
#[case(b"00000000000000000000 000000000000000 00000")]
#[case(b"fn main() { /* test */ }")]
#[case(b"\x01\x00\x00\x00\x00\x00\x01\x01\x01")]
fn test_compression_decompression(#[case] data: &[u8]) {
    let mut cctx = ZStdCctx::new(LEVEL);
    let mut dctx = ZStdDctx::new();

    let mut vec = Vec::with_capacity(64);
    cctx.try_compress_into(data, &mut vec).unwrap();
    let decompressed = dctx.try_decompress(&vec, |_| true).unwrap();

    assert_eq!(decompressed, data);
}
