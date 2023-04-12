use std::{
    borrow::Cow,
    io,
    net::SocketAddr,
};

use galaxy_network::{
    descriptors::{
        CompressionDescriptor,
        PingResponseDescriptor,
    },
    raw::{
        ErrorCode,
        PacketFlags,
        Protocol,
    },
    reader::{
        GalaxyReader,
        Read,
        ReadResult,
    },
    writer::{
        GalaxyWriter,
        Write,
    },
};
use owo_colors::OwoColorize;
use tokio::net::TcpListener;

use crate::{
    config::{
        AuthorizationBackend,
        Config,
    },
    data::user::User,
};

pub async fn create_server<R, D, W, C>(
    writer: &mut GalaxyWriter<W, C>,
    reader: &mut GalaxyReader<R, D>,
    address: SocketAddr,
    user: &mut User,
    flags: PacketFlags,
) -> ReadResult<()>
where
    R: Read,
    W: Write,
{
    let protocol = reader.read_protocol_type(flags).await?;
    match protocol {
        Protocol::Tcp => {
            let port = reader.read_u16().await?;
            if port != 0 && !user.rights.can_select_port(Protocol::Tcp) {
                tracing::error!(
                    "{} Tried to select TCP port: access denied",
                    address.bold()
                );
                writer
                    .server()
                    .write_error(ErrorCode::AccessDenied)
                    .await?;
                return Ok(());
            }

            let listener = match TcpListener::bind(("0.0.0.0", port)).await
            {
                Ok(l) => l,
                Err(e) => {
                    tracing::error!(
                        "{} Failed to bind TCP port: {e}",
                        address.bold()
                    );
                    writer
                        .server()
                        .write_error(ErrorCode::FailedToBindAddress)
                        .await?;
                    return Ok(());
                }
            };

            todo!()
        }

        otherwise => {
            tracing::error!(
                "{} tried to create unimplemented protocol for proxy: \
                 {:?}",
                address.bold(),
                otherwise.bold()
            );
            writer
                .server()
                .write_error(ErrorCode::Unimplemented)
                .await?;
            Ok(())
        }
    }
}

pub async fn authorize_password<R, D, W, C>(
    writer: &mut GalaxyWriter<W, C>,
    reader: &mut GalaxyReader<R, D>,
    address: SocketAddr,
    config: &Config,
    user: &mut User,
) -> io::Result<()>
where
    R: Read,
    W: Write,
{
    match &config.authorization {
        AuthorizationBackend::Password { password } => {
            let supplied_pass = reader.read_string_prefixed().await?;
            if supplied_pass == password.as_str() {
                let new_rights = config.rights.on_password_auth.to_bits();
                user.rights = new_rights;

                tracing::info!(
                    "{} Authorized using password and got the following \
                     rights: {:?}",
                    address.bold(),
                    new_rights.red().bold()
                );

                writer
                    .server()
                    .write_update_rights(new_rights)
                    .await
            } else {
                writer
                    .server()
                    .write_error(ErrorCode::AccessDenied)
                    .await
            }
        }
        AuthorizationBackend::Database { .. } => {
            tracing::error!(
                "{} Failed to authorize using password: working in {} \
                 mode",
                address.bold(),
                "`password`".bold()
            );
            writer
                .server()
                .write_error(ErrorCode::Unsupported)
                .await
        }
    }
}

pub async fn ping<W: Write, C>(
    writer: &mut GalaxyWriter<W, C>,
    address: SocketAddr,
    config: &Config,
) -> io::Result<()> {
    tracing::info!("{} Requested ping", address.bold());
    writer
        .server()
        .write_ping(&PingResponseDescriptor {
            buffer_read: config.server.buffering.read,
            server_name: Cow::Borrowed(&config.server.name),
            compression: CompressionDescriptor {
                level: config.compression.level,
                algorithm: config.compression.algorithm,
            },
        })
        .await?;
    Ok(())
}
