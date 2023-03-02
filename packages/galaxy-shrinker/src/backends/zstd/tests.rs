use rstest::rstest;

use super::{
    cctx::ZStdCctx,
    dctx::ZStdDctx,
};
use crate::interface::*;

#[rstest]
#[case(b"000010000000000001000000000001000000")]
#[case(b"Hello world\nHell0 world\nHello world\nHello world\nHello world\n")]
#[case(
    b"#[case(0, 0)]
#[case(1, 1)]
#[case(2, 1)]
#[case(3, 2)]
#[case(4, 3)]"
)]
#[case(
    b"You can also inject values in some other ways.
    For instance, you can create a set of tests by simply providing the
    injected values for each case: rstest will generate
    an independent test for each case."
)]
fn test_zstd_compress_decompress(#[case] in_: &[u8]) {
    let mut cctx = ZStdCctx::new(20.try_into().unwrap());
    let mut dctx = ZStdDctx::new();
    let mut dst = Vec::with_capacity(in_.len());

    let comp_size = cctx.try_compress(in_, &mut dst).unwrap();
    let orig = dctx
        .try_decompress(&dst, |size| {
            Vec::with_capacity(size.unwrap().get()).into()
        })
        .unwrap();

    assert_eq!(orig, in_);
}
