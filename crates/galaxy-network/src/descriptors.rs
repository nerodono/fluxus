use std::{
    borrow::Cow,
    num::{
        NonZeroU16,
        NonZeroU8,
        NonZeroUsize,
    },
};

use crate::raw::CompressionAlgorithm;

#[derive(Debug)]
pub struct CompressionDescriptor {
    pub algorithm: CompressionAlgorithm,
    pub level: NonZeroU8,
}

#[derive(Debug)]
pub struct CreateServerResponseDescriptor {
    pub port: Option<NonZeroU16>,
}

#[derive(Debug)]
pub struct PingResponseDescriptor<'a> {
    pub compression: CompressionDescriptor,
    pub server_name: Cow<'a, str>,
    pub buffer_read: NonZeroUsize,
}
