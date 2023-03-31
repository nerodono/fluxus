use std::num::{
    NonZeroU8,
    NonZeroUsize,
};

use crate::raw::CompressionAlgorithm;

pub struct CompressionDescriptor {
    pub algorithm: CompressionAlgorithm,
    pub level: NonZeroU8,
}

pub struct PingResponseDescriptor<'a> {
    pub compression: CompressionDescriptor,
    pub server_name: &'a str,
    pub buffer_read: NonZeroUsize,
}
