use std::borrow::Cow;

use flux_common::Rights;
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
    types::pkt_base::PktFlags,
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
    pub state: &'r mut ConnectionState<'cfg>,

    pub flags: PktFlags,
    pub reader: MasterServerReader<'r, R>,
    pub writer: &'r mut MasterServerWriter<W>,
}

// Server creation
impl<'r, 'cfg, R: RawRead, W: RawWrite> Atom<'r, 'cfg, R, W> {
    #[cfg(feature = "tcp")]
    pub async fn create_tcp(mut self) -> TcpFluxResult<()> {
        let request = self
            .reader
            .read_create_tcp_request(self.flags)
            .await?;

        self.state
            .require_rights(Rights::CAN_CREATE_TCP_PROXY)?;
        self.state
            .require_rights(if request.specific_port.is_some() {
                Rights::CAN_PICK_TCP_PORT
            } else {
                Rights::empty()
            })?;

        Ok(())
    }

    #[cfg(feature = "http")]
    pub async fn create_http(self) -> TcpFluxResult<()> {
        todo!()
    }
}

// Service functions (information retrieval, for example)
impl<'r, 'cfg, R: RawRead, W: RawWrite> Atom<'r, 'cfg, R, W> {
    /// Try authenticate the user
    ///
    /// # Errors
    pub async fn authenticate(self) -> TcpFluxResult<()> {
        todo!()
    }

    /// Sends information about the server to the client
    pub async fn req_info(self) -> TcpFluxResult<()> {
        tracing::info!("{} server information request", self.state.user);
        self.writer
            .write_info(InfoPayload {
                server_name: Cow::Borrowed(&self.state.config.server.name),
            })
            .await
            .map_err(TcpFluxError::Io)
    }

    pub async fn disconnect(self) -> TcpFluxResult<()> {
        todo!()
    }
}

#[cfg(not(all(feature = "tcp", feature = "http")))]
impl<'r, 'cfg, R: RawRead, W: RawWrite> Atom<'r, 'cfg, R, W> {
    #[cfg(not(feature = "tcp"))]
    pub async fn create_tcp(self) -> TcpFluxResult<()> {
        self.opted_out("TCP proxy").await
    }

    #[cfg(not(feature = "http"))]
    pub async fn create_http(self) -> TcpFluxResult<()> {
        self.opted_out("HTTP proxy").await
    }

    async fn opted_out(self, name: &'static str) -> TcpFluxResult<()> {
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
