use std::sync::Arc;

use color_eyre::eyre::{
    self,
    Context,
};
use tcp_flux::{
    connection::any::ConnectionType,
    listener::Listener,
};
use tokio::net::TcpListener;

use crate::config::root::Config;

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
        match connection.type_ {
            ConnectionType::Flow { id } => {
                tracing::info!("{} connected to the flow {id}", connection.address);
                todo!()
            }

            ConnectionType::Master => {
                tracing::info!("{} connected as the master", connection.address);
                todo!()
            }
        }
    }
}
