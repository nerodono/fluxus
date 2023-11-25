use std::borrow::Cow;

use tcp_flux::{
    connection::{
        master::{
            payloads::info::InfoPayload,
            reader::server::MasterServerReader,
            writer::server::MasterServerWriter,
        },
        traits::{
            RawRead,
            RawWrite,
        },
    },
    types::{
        error_code::ErrorCode,
        pkt_base::PktFlags,
    },
};

use super::connection::ConnectionState;
use crate::protocols::tcp_flux::error::{
    TcpFluxError,
    TcpFluxResult,
};

/// Indivisible scope of connection: actual packet handling
///
/// ```text
///  Connection*
/// ------------>
/// |----|------|
///  ^^^^ ^^^^^^
///  Atom  Atom
/// ```
/// * Connection here is an actual connection, not the
///   **state of connection**
pub struct Atom<'r, 'cfg, R, W> {
    state: &'r mut ConnectionState<'cfg>,

    flags: PktFlags,
    writer: &'r mut MasterServerWriter<W>,
    reader: MasterServerReader<'r, R>,
}

// Server creation
impl<'r, 'cfg, R: RawRead, W: RawWrite> Atom<'r, 'cfg, R, W> {
    #[cfg(feature = "tcp")]
    pub async fn create_tcp(&mut self) -> TcpFluxResult<()> {
        let request = self
            .reader
            .read_create_tcp_request(self.flags)
            .await?;
        todo!()
    }

    #[cfg(feature = "http")]
    pub async fn create_http(&mut self) -> TcpFluxResult<()> {
        todo!()
    }
}

impl<'r, 'cfg, R: RawRead, W: RawWrite> Atom<'r, 'cfg, R, W> {
    #[cfg(not(feature = "tcp"))]
    pub async fn create_tcp(&mut self) -> TcpFluxResult<()> {
        self.opted_out("TCP proxy").await
    }

    #[cfg(not(feature = "http"))]
    pub async fn create_http(&mut self) -> TcpFluxResult<()> {
        self.opted_out("HTTP proxy").await
    }

    #[allow(dead_code)]
    async fn opted_out(&mut self, name: &'static str) -> TcpFluxResult<()> {
        tracing::error!(
            "{} tried to call opted-out functionality: {name}",
            self.state.user
        );
        self.writer
            .write_error(ErrorCode::OptedOut)
            .await
            .map_err(TcpFluxError::Io)
    }
}

// Service functions (information retrieval, for example)
impl<'r, 'cfg, R: RawRead, W: RawWrite> Atom<'r, 'cfg, R, W> {
    /// Try authenticate the user
    ///
    /// # Errors
    pub async fn authenticate(&mut self) -> TcpFluxResult<()> {
        todo!()
    }

    /// Sends information about the server to the client
    pub async fn req_info(&mut self) -> TcpFluxResult<()> {
        tracing::info!("{} server information request", self.state.user);
        self.writer
            .write_info(InfoPayload {
                server_name: Cow::Borrowed(&self.state.config.server.name),
            })
            .await
            .map_err(TcpFluxError::Io)
    }

    pub async fn disconnect(&mut self) -> TcpFluxResult<()> {
        todo!()
    }
}

impl<'r, 'cfg, R, W> Atom<'r, 'cfg, R, W> {
    pub fn new(
        state: &'r mut ConnectionState<'cfg>,
        reader: MasterServerReader<'r, R>,
        writer: &'r mut MasterServerWriter<W>,
        flags: PktFlags,
    ) -> Self {
        Self {
            state,
            reader,
            writer,
            flags,
        }
    }
}
