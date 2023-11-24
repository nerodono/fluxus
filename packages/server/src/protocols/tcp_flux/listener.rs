use std::sync::Arc;

use color_eyre::eyre::{
    self,
    Context,
};
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
use tokio::net::TcpListener;

use crate::{
    config::root::Config,
    protocols::tcp_flux::master::{
        connection::{
            ConnectionState,
            Sides,
        },
        router::Router,
    },
};

fn grab_port_from(listener: &TcpListener) -> eyre::Result<u16> {
    listener
        .local_addr()
        .wrap_err("failed to fetch bound port")
        .map(|a| a.port())
}

pub async fn run(config: Arc<Config>) -> eyre::Result<()> {
    let listener = Listener::bind(config.protocols.tcp_flux.listen).await?;
    let bound_port = grab_port_from(listener.inner_ref())?;

    tracing::info!("tcpflux is listening on :{bound_port}");
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
                let state =
                    ConnectionState::new(Arc::clone(&config), connection.address);
                Router::new(
                    state,
                    Sides {
                        reader: MasterReader::new(reader),
                        writer: MasterServerWriter::new(writer),
                    },
                )
                .serve()
                .await
            }
        };

        if let Err(e) = execution_result {
            tracing::error!("{} disconnected ({e})", connection.address);
        }
    }
}
