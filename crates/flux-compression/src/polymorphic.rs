#![allow(clippy::missing_const_for_fn)]
use enum_dispatch::enum_dispatch;

pub(crate) use crate::zstd::{
    ZStdCctx,
    ZStdDctx,
};

#[enum_dispatch(Compressor)]
pub enum PolyCctx {
    ZStd(ZStdCctx),
}

#[enum_dispatch(Decompressor)]
pub enum PolyDctx {
    ZStd(ZStdDctx),
}

fn _test_compressor(_: impl crate::traits::Compressor) {}
fn _test_decompressor(_: impl crate::traits::Decompressor) {}

fn _test_impls(cctx: PolyCctx, dctx: PolyDctx) {
    _test_compressor(cctx);
    _test_decompressor(dctx);
}
