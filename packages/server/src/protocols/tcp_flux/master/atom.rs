use std::borrow::Cow;

use flux_common::Rights;
use tcp_flux::{
    connection::{
        master::{
            payloads::{
                create_tcp_request::CreateTcpRequest,
                info::InfoPayload,
            },
            writer::server::MasterServerWriter,
        },
        traits::RawWrite,
    },
    types::error_code::ErrorCode,
};

use super::connection::ConnectionState;
use crate::protocols::tcp_flux::{
    error::{
        TcpFluxError,
        TcpFluxResult,
    },
    layered::{
        filters::enough_rights::{
            enough_rights,
            enough_rights_lazy,
        },
        handler::{
            CombineExt,
            FilteredExt,
            Handler,
            HandlerF,
        },
    },
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
pub struct Atom<'r, 'cfg, W> {
    state: &'r mut ConnectionState<'cfg>,
    writer: &'r mut MasterServerWriter<W>,
}

// Server creation
impl<'r, 'cfg, W: RawWrite> Atom<'r, 'cfg, W> {
    #[cfg(feature = "tcp")]
    pub fn create_tcp(&mut self) -> impl HandlerF<CreateTcpRequest> + '_ {
        Handler(|payload| async move { todo!() }).filter(
            (
                enough_rights(self.state.user.rights, Rights::CAN_CREATE_TCP_PROXY),
                enough_rights_lazy::<CreateTcpRequest, _>(
                    self.state.user.rights,
                    |p| {
                        if p.specific_port.is_some() {
                            Rights::CAN_PICK_TCP_PORT
                        } else {
                            Rights::empty()
                        }
                    },
                ),
            )
                .combined(),
        )
    }

    #[cfg(feature = "http")]
    pub async fn create_http(&mut self) -> TcpFluxResult<()> {
        todo!()
    }
}

impl<'r, 'cfg, W: RawWrite> Atom<'r, 'cfg, W> {
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
impl<'r, 'cfg, W: RawWrite> Atom<'r, 'cfg, W> {
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

impl<'r, 'cfg, W> Atom<'r, 'cfg, W> {
    pub fn new(
        state: &'r mut ConnectionState<'cfg>,
        writer: &'r mut MasterServerWriter<W>,
    ) -> Self {
        Self { state, writer }
    }
}
