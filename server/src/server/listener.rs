use std::{
    fmt::Display,
    net::SocketAddr,
    sync::Arc,
};

use flux_tcp::raw::ConnectProtocol;
use owo_colors::OwoColorize;
use tokio::{
    io::AsyncReadExt,
    net::{
        TcpListener,
        TcpStream,
    },
};

use crate::{
    config::Config,
    data::server_map::ServerMap,
    server::control,
};

async fn dispatch_protocols(
    config: Arc<Config>,
    server_map: ServerMap,
    mut stream: TcpStream,
    address: SocketAddr,
) -> eyre::Result<()> {
    let raw_protocol_type = stream.read_u8().await?;
    let Ok(protocol_type) = ConnectProtocol::try_from(raw_protocol_type) else {
        tracing::error!(
            "{} tried to connect to the wrong protocol: 0x{raw_protocol_type:x}",
            address.bold()
        );
        return Ok(());
    };

    let (raw_reader, raw_writer) = stream.split();

    match protocol_type {
        ConnectProtocol::Control => {
            tracing::info!(
                "{} connected to the control flux",
                address.bold()
            );

            #[rustfmt::skip]
            let mut connection = control::connection::Connection::new(
                raw_reader,
                raw_writer,
                address,
                config,
                server_map
            );

            display_disconnection(connection.serve().await, address);
        }

        ConnectProtocol::Flow => {
            tracing::info!("{} connected to the flow flux", address.bold());
            todo!();
        }
    }

    Ok(())
}

fn display_disconnection<T, E: Display>(
    result: Result<T, E>,
    address: SocketAddr,
) {
    let bold_addr = address.bold();
    if let Err(e) = result {
        tracing::error!("{bold_addr} disconnected with an error: {e}");
    } else {
        tracing::info!("{bold_addr} disconnected without any errors");
    }
}

pub async fn run_flux_listener(config: Arc<Config>) -> eyre::Result<()> {
    let listener = TcpListener::bind(config.server.listen).await?;
    let bound_on = listener.local_addr()?;

    tracing::info!(
        "started {} listener on {}",
        "flux".bold().green(),
        bound_on.bold()
    );

    let server_map = ServerMap::new();

    loop {
        let (stream, address) = listener.accept().await?;

        let server_map = server_map.clone();
        let config = Arc::clone(&config);
        tokio::spawn(dispatch_protocols(config, server_map, stream, address));
    }
}
