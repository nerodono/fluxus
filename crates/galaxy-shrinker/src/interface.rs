use std::num::NonZeroUsize;

pub struct CompressorStub;
pub struct DecompressorStub;

pub trait Compressor: Send {
    fn try_compress(&mut self, src: &[u8]) -> Option<Vec<u8>>;
}

pub trait Decompressor: Send {
    fn try_get_decompressed_size(&self, src: &[u8]) -> Option<NonZeroUsize>;
    fn try_decompress(
        &mut self,
        src: &[u8],
        preallocate: usize,
    ) -> Option<Vec<u8>>;
}

pub trait CStub {}
pub trait DStub {}

impl CStub for CompressorStub {}
impl DStub for DecompressorStub {}

impl<T: CStub + Send> Compressor for T {
    #[track_caller]
    fn try_compress(&mut self, _src: &[u8]) -> Option<Vec<u8>> {
        unimplemented!()
    }
}
impl<T: DStub + Send> Decompressor for T {
    #[track_caller]
    fn try_get_decompressed_size(&self, _src: &[u8]) -> Option<NonZeroUsize> {
        unimplemented!()
    }

    #[track_caller]
    fn try_decompress(
        &mut self,
        _src: &[u8],
        _prallocate: usize,
    ) -> Option<Vec<u8>> {
        unimplemented!()
    }
}
