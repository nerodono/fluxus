use std::num::{
    NonZeroU16,
    NonZeroU8,
    NonZeroUsize,
};

use crate::raw::CompressionAlgorithm;

pub struct CompressionDescriptor {
    pub algorithm: CompressionAlgorithm,
    pub level: NonZeroU8,
}

pub struct CreateServerResponseDescriptor {
    pub port: Option<NonZeroU16>,
}

pub struct PingResponseDescriptor<'a> {
    pub compression: CompressionDescriptor,
    pub server_name: &'a str,
    pub buffer_read: NonZeroUsize,
}
