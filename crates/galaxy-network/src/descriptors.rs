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
pub enum CreateServerRequestDescriptor<'a> {
    Http { endpoint: &'a [u8] },
    Tcp { port: Option<NonZeroU16> },
}

#[derive(Debug)]
pub enum CreateServerResponseDescriptor {
    Http { endpoint: Option<Vec<u8>> },
    Tcp { port: Option<NonZeroU16> },
}

#[derive(Debug)]
pub struct CompressionDescriptor {
    pub algorithm: CompressionAlgorithm,
    pub level: NonZeroU8,
}

#[derive(Debug)]
pub struct PingResponseDescriptor<'a> {
    pub compression: CompressionDescriptor,
    pub server_name: Cow<'a, str>,
    pub buffer_read: NonZeroUsize,
}

impl CreateServerResponseDescriptor {
    #[track_caller]
    pub fn unwrap_tcp_port(self) -> Option<NonZeroU16> {
        match self {
            Self::Tcp { port } => port,
            Self::Http { .. } => wrong_variant(),
        }
    }

    #[track_caller]
    pub fn unwrap_http_endpoint(self) -> Option<Vec<u8>> {
        match self {
            Self::Http { endpoint } => endpoint,
            Self::Tcp { .. } => wrong_variant(),
        }
    }
}

#[cold]
#[track_caller]
fn wrong_variant() -> ! {
    panic!("Wrong variant picked: end game for you :)")
}