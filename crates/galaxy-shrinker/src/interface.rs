pub struct CompressorStub;
pub struct DecompressorStub;

pub trait CStub {}
pub trait DStub {}

pub trait Compressor {}

pub trait Decompressor {}

impl CStub for CompressorStub {}
impl DStub for DecompressorStub {}

impl<T: CStub> Compressor for T {}
impl<T: DStub> Decompressor for T {}
