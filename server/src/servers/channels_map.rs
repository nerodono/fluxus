use rustc_hash::FxHashMap;
use tokio::sync::mpsc;

use crate::{
    error::{
        NonCriticalError,
        NonCriticalResult,
    },
    utils::return_id::ReturnId,
};

/// Simple mapping for id:channel functionality
pub struct ChannelsMap<T> {
    channels: FxHashMap<u16, mpsc::UnboundedSender<T>>,
}

impl<T> Default for ChannelsMap<T> {
    fn default() -> Self {
        Self {
            channels: FxHashMap::default(),
        }
    }
}

impl<T> From<FxHashMap<u16, mpsc::UnboundedSender<T>>> for ChannelsMap<T> {
    fn from(value: FxHashMap<u16, mpsc::UnboundedSender<T>>) -> Self {
        Self { channels: value }
    }
}

impl<T> ChannelsMap<T> {
    pub fn insert(&mut self, id: u16, chan: mpsc::UnboundedSender<T>) {
        self.channels.insert(id, chan);
    }

    pub fn send_command(&self, id: u16, command: T) -> NonCriticalResult<()> {
        let channel = self.channels.get(&id).ok_or(
            NonCriticalError::ClientIsNotFound {
                id,
                chan_closed: false,
            },
        )?;

        channel.send(command).map_err(|_| {
            NonCriticalError::ClientIsNotFound {
                id,
                chan_closed: true,
            }
        })
    }

    pub fn remove(
        &mut self,
        id: u16,
    ) -> NonCriticalResult<ReturnId<u16, mpsc::UnboundedSender<T>>> {
        self.channels
            .remove(&id)
            .map(|channel| ReturnId::new(id, channel))
            .ok_or(NonCriticalError::ClientIsNotFound {
                id,
                chan_closed: true,
            })
    }
}
