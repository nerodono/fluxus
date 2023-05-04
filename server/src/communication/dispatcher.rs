use std::{
    net::SocketAddr,
    num::NonZeroU16,
};

use galaxy_network::{
    shrinker::interface::Compressor,
    writer::{
        GalaxyWriter,
        Write,
    },
};

use crate::{
    data::{
        commands::master::MasterCommand,
        proxy::Proxy,
    },
    error::ProcessResult,
};

pub struct CommandDispatcher {
    pub address: SocketAddr,
    threshold: Option<NonZeroU16>,
}

impl CommandDispatcher {
    pub const fn new(
        address: SocketAddr,
        threshold: Option<NonZeroU16>,
    ) -> Self {
        Self { address, threshold }
    }

    pub async fn dispatch<W, C>(
        &self,
        writer: &mut GalaxyWriter<W, C>,
        proxy: &mut Proxy,
        command: MasterCommand,
    ) -> ProcessResult<bool>
    where
        W: Write,
        C: Compressor,
    {
        match command {
            #[cfg(feature = "http")]
            MasterCommand::Http(http_cmd) => {
                todo!();
            }

            #[cfg(feature = "tcp")]
            MasterCommand::Tcp(tcp_cmd) => {
                let server = unsafe { proxy.data.unwrap_tcp_unchecked() };
                super::tcp::handle_command(
                    &proxy.pool,
                    writer,
                    server,
                    self.threshold,
                    tcp_cmd,
                )
                .await
            }
        }
    }
}
