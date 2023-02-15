use std::io;

use mid_net::{
    prelude::WriterUnderlyingExt,
    proto::ProtocolError,
    writer::MidWriter,
};

use super::message::SlaveMessage;
use crate::tcp::state::State;

/// Utility function for communication with the slaves.
/// Contains shared functionality of some methods
pub async fn send_slave_message_to<W, C>(
    writer: &mut MidWriter<W, C>,
    id: u16,
    state: &mut State,
    message: SlaveMessage,
) -> io::Result<()>
where
    W: WriterUnderlyingExt,
{
    match state.server {
        Some(ref mut server) => {
            if server.send_message(id, message).await.is_err() {
                writer
                    .server()
                    .write_failure(ProtocolError::ClientDoesNotExists)
                    .await
            } else {
                Ok(())
            }
        }

        None => {
            writer
                .server()
                .write_failure(ProtocolError::ServerIsNotCreated)
                .await
        }
    }
}
