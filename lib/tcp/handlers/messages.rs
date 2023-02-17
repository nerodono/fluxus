use std::{
    io,
    net::SocketAddr,
};

use mid_net::{
    prelude::{
        impl_::interface::ICompressor,
        *,
    },
    proto::ProtocolError,
    utils::FancyUtilExt,
};

use super::message_types::SlaveMessage;
use crate::tcp::state::State;

/// Triggered on connect message
pub async fn on_connected<W, C>(
    writer: &mut MidWriter<W, C>,
    id: u16,
    tx: flume::Sender<SlaveMessage>,
    state: &mut State,
    address: &SocketAddr,
) -> io::Result<()>
where
    W: WriterUnderlyingExt,
{
    if let Some(ref mut server) = state.server {
        server.create_client(id, tx);
        writer.server().write_connected(id).await
    } else {
        tracing::error!(
            %address,
            "Failed to register connected user: server does not exists"
        );
        Ok(())
    }
}

/// Triggered on forward message
pub async fn on_forward<W, C>(
    writer: &mut MidWriter<W, C>,
    id: u16,
    data: Vec<u8>,
    threshold: usize,
) -> io::Result<()>
where
    W: WriterUnderlyingExt,
    C: ICompressor,
{
    writer
        .write_forward(
            id,
            &data,
            ForwardCompression::Compress {
                with_threshold: threshold,
            },
        )
        .await
        .unitize_io()
}

/// Triggered on disconnected message
pub async fn on_disconnected<W, C>(
    writer: &mut MidWriter<W, C>,
    state: &mut State,
    id: u16,
) -> io::Result<()>
where
    W: WriterUnderlyingExt,
{
    if let Some(ref mut server) = state.server {
        server
            .unlisten_slave(id)
            .await
            .unwrap_or_default();
        writer.write_disconnected(id).await
    } else {
        tracing::error!("Failed to disconnect client: server was not created");
        Ok(())
    }
}

/// Triggered when server was shut down
pub async fn on_shutdown<W, C>(
    writer: &mut MidWriter<W, C>,
    state: &mut State,
) -> io::Result<()>
where
    W: WriterUnderlyingExt,
{
    state.server = None;
    writer
        .server()
        .write_failure(ProtocolError::ServerWasShutDown)
        .await
}
