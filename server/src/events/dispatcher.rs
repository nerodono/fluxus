use std::{
    io,
    net::SocketAddr,
};

use cfg_if::cfg_if;
use galaxy_network::{
    raw::ErrorCode,
    shrinker::interface::Compressor,
    writer::{
        GalaxyWriter,
        Write,
    },
};

cfg_if! {
    if #[cfg(feature = "http")] {
        use crate::events::http::handle_http_command;
    }
}

cfg_if! {
    if #[cfg(feature = "galaxy")] {
        use super::tcp::handle_tcp_command;
    }
}

use crate::{
    config::Config,
    data::{
        commands::base::MasterCommand,
        user::User,
    },
    utils::compiler::unlikely,
};

pub async fn dispatch_command<W, C>(
    writer: &mut GalaxyWriter<W, C>,
    address: SocketAddr,
    user: &mut User,
    command: MasterCommand,
    config: &Config,
) -> io::Result<()>
where
    W: Write,
    C: Compressor,
{
    // SAFETY: safe since command may arrive only if recv
    // channel is alive. recv channel is alive only if
    // `user.proxy` is `Some`.
    #[cfg(any(feature = "galaxy", feature = "http"))]
    {
        // Scope limitation is needed to revoke &mut borrow of the
        // proxy variant
        let stop = {
            let proxy = unsafe { user.proxy.as_mut().unwrap_unchecked() };

            // `proxy.data` unwraps SAFETY: safe since sending permits
            // are statically checked
            match command {
                #[cfg(feature = "galaxy")]
                MasterCommand::Tcp(tcp) => {
                    let server = unsafe { proxy.data.unwrap_tcp_unchecked() };
                    handle_tcp_command(server, address, tcp, writer, config)
                        .await
                }

                #[cfg(feature = "http")]
                MasterCommand::Http(http) => {
                    let server =
                        unsafe { proxy.data.unwrap_http_unchecked() };
                    handle_http_command(server, address, http, writer, config)
                        .await
                }
            }
        }?;
        if unlikely(stop) {
            user.proxy = None;
            writer
                .server()
                .write_error(ErrorCode::ServerStopped)
                .await?;
        }
    }

    Ok(())
}
