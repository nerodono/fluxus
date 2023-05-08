use std::mem;

use super::channels_map::ChannelsMap;
use crate::{
    data::commands::http::{
        HttpServerRequest,
        HttpSlaveCommand,
    },
    utils::features::http::HttpChannel,
};

pub struct HttpServer {
    pub endpoint: Vec<u8>,
    pub server_channel: HttpChannel,
    pub channels: ChannelsMap<HttpSlaveCommand>,
}

impl HttpServer {
    pub fn new(endpoint: Vec<u8>, server_channel: HttpChannel) -> Self {
        Self {
            endpoint,
            server_channel,
            channels: ChannelsMap::default(),
        }
    }
}

impl Drop for HttpServer {
    fn drop(&mut self) {
        _ = self
            .server_channel
            .send(HttpServerRequest::Unbind {
                endpoint: mem::take(&mut self.endpoint),
            });
    }
}
