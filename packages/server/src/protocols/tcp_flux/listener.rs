use std::sync::Arc;

use color_eyre::eyre;
use tcp_flux::{
    connection::{
        any::ConnectionType,
        master::{
            reader::common::MasterReader,
            writer::server::MasterServerWriter,
        },
    },
    listener::Listener,
};

use super::master::network::connection::ConnectionState;
use crate::{
    config::root::Config,
    protocols::tcp_flux::master::handler::handle_connection,
};

pub async fn run(config: Arc<Config>) -> eyre::Result<()> {
    let listener = Listener::bind(config.server.protocols.tcp_flux.listen).await?;
    let bound_address = listener.inner_ref().local_addr()?;

    tracing::info!("tcpflux is listening on {bound_address}");
    loop {
        let connection = listener.next_connection().await?;
        let (reader, writer) = connection.socket.into_split();

        let execution_result = match connection.type_ {
            ConnectionType::Flow { id } => {
                tracing::info!("{} connected to the flow {id}", connection.address);
                todo!()
            }

            ConnectionType::Master => {
                tracing::info!("{} connected as the master", connection.address);
                let state = ConnectionState::new(&config, connection.address);
                handle_connection(
                    MasterReader::new(reader),
                    MasterServerWriter::new(writer),
                    state,
                )
                .await
            }
        };

        if let Err(e) = execution_result {
            tracing::error!("{} disconnected ({e})", connection.address);
        }
    }
}
