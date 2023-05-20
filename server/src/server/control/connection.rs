use std::{
    io,
    net::SocketAddr,
    sync::Arc,
};

use flux_tcp::{
    control::{
        reader::ControlReader,
        writer::ControlWriter,
    },
    error::{
        ControlReadError,
        ControlReadResult,
    },
    raw::{
        NetworkError,
        PacketType,
        Rights,
    },
    traits::{
        ComposeRead,
        ComposeWrite,
    },
};
use owo_colors::OwoColorize;
use tokio::{
    net::TcpListener,
    sync::mpsc,
};

use crate::{
    communication::control::ControlCommand,
    config::Config,
    data::{
        server::Server,
        server_map::ServerMap,
        user::User,
    },
    error::{
        NonCriticalError,
        ProcessError,
        ProcessResult,
    },
    server::slave::listener::run_tcp_slave_listener,
    traits::FromConfig,
    utils::{
        compression::{
            create_cctx,
            create_dctx,
        },
        shutdown_token::shutdown_token,
    },
};

pub struct Connection<R, W> {
    reader: ControlReader<R>,
    writer: ControlWriter<W>,

    user: User,
    config: Arc<Config>,

    tx: mpsc::Sender<ControlCommand>,
    rx: mpsc::Receiver<ControlCommand>,

    map: ServerMap,
}

impl<R, W> Connection<R, W>
where
    R: ComposeRead,
    W: ComposeWrite,
{
    async fn create_tcp_server(&mut self) -> ProcessResult<()> {
        let port = self.reader.read_u16().await?;
        if !self.user.rights.contains(Rights::CREATE_TCP) {
            return Err(
                NonCriticalError::AccessDenied("create tcp server").into()
            );
        }

        if port != 0 && !self.user.rights.contains(Rights::SELECT_TCP_PORT) {
            return Err(
                NonCriticalError::AccessDenied("select tcp port").into()
            );
        }

        let (bound_on, listener) = match TcpListener::bind(("0.0.0.0", port))
            .await
            .and_then(|l| Ok((l.local_addr()?, l)))
        {
            Ok(l) => l,
            Err(e) => {
                return Err(NonCriticalError::FailedToBindAddress {
                    port,
                    error: e,
                }
                .into())
            }
        };

        let server = Arc::new(Server::new(
            self.user.address(),
            self.config.server.backlog.get(),
        ));
        self.map
            .map_address(self.user.address(), Arc::clone(&server))
            .await;

        let (shutdown_reader, shutdown_writer) = shutdown_token();

        self.user
            .set_server(Some((shutdown_writer, self.map.clone())));
        tokio::spawn(run_tcp_slave_listener(
            create_cctx,
            create_dctx,
            server,
            Arc::clone(&self.config),
            self.user.address(),
            shutdown_reader,
            listener,
            self.tx.clone(),
        ));

        self.writer
            .server()
            .write_tcp_server(bound_on.port())
            .await?;

        Ok(())
    }

    async fn authorize_password(&mut self) -> ProcessResult<()> {
        let bytes = self.reader.read_bytes_prefixed().await?;
        let supplied_password = String::from_utf8_lossy(&bytes);
        if let Some(ref password) = self.config.server.universal_password {
            if &supplied_password == password {
                let new_rights = Rights::from_config(
                    &self.config.rights.on_universal_password,
                );
                self.user.rights = new_rights;

                self.writer
                    .server()
                    .write_update_rights(new_rights)
                    .await?;
                tracing::info!(
                    "{} authorized and got the following rights: \
                     {new_rights:?}",
                    self.user
                );

                Ok(())
            } else {
                Err(NonCriticalError::WrongPassword.into())
            }
        } else {
            Err(NonCriticalError::Disabled("password authorization").into())
        }
    }
}

impl<R, W> Connection<R, W>
where
    R: ComposeRead,
    W: ComposeWrite,
{
    async fn handle_command(
        &mut self,
        command: ControlCommand,
    ) -> io::Result<bool> {
        match command {
            ControlCommand::Connected => {
                self.writer.server().write_connected().await?;
            }

            ControlCommand::Shutdown => {
                self.writer
                    .server()
                    .write_error(NetworkError::Shutdown)
                    .await?;
                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl<R, W> Connection<R, W>
where
    R: ComposeRead,
    W: ComposeWrite,
{
    pub async fn serve(&mut self) -> ControlReadResult<()> {
        self.writer
            .server()
            .write_hello(
                self.user.address().port(),
                self.config
                    .buffering
                    .read
                    .flow
                    .get()
                    .try_into()
                    .unwrap_or(u16::MAX),
            )
            .await?;

        loop {
            let packet_type;
            tokio::select! {
                // biased checking is not a problem at all
                // because everything it does is handling two possible types:
                // Connected and Shutdown, they're not so frequent (or not frequent at all)
                // especially when compared to the network rate. Basically this saves just
                // some time on not generating the random numbers, that's it.
                biased;

                command = self.rx.recv() => {
                    let Some(command) = command else {
                        break;
                    };
                    let reset_server = self.handle_command(command).await?;
                    if reset_server {
                        self.user.set_server(None);
                    }

                    continue;
                }

                pkt_ty = self.reader.read_u8() => {
                    let pkt_ty = pkt_ty?;
                    packet_type = if let Ok(p) = PacketType::try_from(pkt_ty) {
                        p
                    } else {
                        return Err(ControlReadError::UnknownPacket(pkt_ty));
                    };
                }
            }

            let result = match packet_type {
                PacketType::AuthorizePassword => {
                    self.authorize_password().await
                }

                PacketType::CreateTcpServer => self.create_tcp_server().await,

                pty => {
                    tracing::error!(
                        "{} sent unexpected packet to the control flow: \
                         {pty:?}",
                        self.user,
                    );

                    break;
                }
            };

            match result {
                Err(ProcessError::Control(control)) => return Err(control),
                Err(ProcessError::NonCritical(non_critical)) => {
                    tracing::error!(
                        "{} non-critical error reported: {}",
                        non_critical.bold().red(),
                        self.user
                    );
                    let network_error: NetworkError = non_critical.into();

                    _ = self
                        .writer
                        .server()
                        .write_error(network_error)
                        .await;
                    continue;
                }

                Err(ProcessError::Io(io)) => return Err(io.into()),

                Ok(()) => {}
            }
        }
        Ok(())
    }
}

impl<R, W> Connection<R, W>
where
    R: ComposeRead,
    W: ComposeWrite,
{
    pub fn new(
        reader: R,
        writer: W,
        address: SocketAddr,
        config: Arc<Config>,
        map: ServerMap,
    ) -> Self {
        let rights = Rights::from_config(&config.rights.on_connect);
        let (reader, writer) = (
            ControlReader::with_capacity(
                reader,
                config.buffering.read.control.get(),
            ),
            ControlWriter::new(writer),
        );

        let (tx, rx) = mpsc::channel(config.buffering.channels.control.get());

        Self {
            reader,
            writer,
            config,
            tx,
            rx,
            map,
            user: User::new(address, rights),
        }
    }
}
