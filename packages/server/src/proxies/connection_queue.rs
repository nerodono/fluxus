use std::{
    net::SocketAddr,
    sync::Arc,
};

use dashmap::{
    mapref::entry::Entry,
    DashMap,
};

pub struct NoSuchQueue;
pub struct QueueAlreadyExists;

pub struct ConnectionQueue<T> {
    map: Arc<DashMap<SocketAddr, Vec<T>>>,
}

impl<T> ConnectionQueue<T> {
    pub fn create_queue(&self, key: SocketAddr) -> Result<(), QueueAlreadyExists> {
        match self.map.entry(key) {
            Entry::Occupied(_) => Err(QueueAlreadyExists),
            Entry::Vacant(vacant) => {
                vacant.insert(Vec::with_capacity(4));
                Ok(())
            }
        }
    }

    pub fn drop_queue(&self, key: &SocketAddr) -> Result<(), NoSuchQueue> {
        if self.map.remove(key).is_some() {
            Ok(())
        } else {
            Err(NoSuchQueue)
        }
    }
}

impl<T> ConnectionQueue<T> {
    pub fn push(&self, key: SocketAddr, item: T) -> Result<(), NoSuchQueue> {
        self.map
            .get_mut(&key)
            .ok_or(NoSuchQueue)?
            .push(item);

        Ok(())
    }

    pub fn pop(&self, key: &SocketAddr) -> Option<T> {
        self.map.get_mut(key)?.pop()
    }
}

impl<T> Default for ConnectionQueue<T> {
    fn default() -> Self {
        Self {
            map: Arc::default(),
        }
    }
}

impl<T> Clone for ConnectionQueue<T> {
    fn clone(&self) -> Self {
        Self {
            map: Arc::clone(&self.map),
        }
    }
}
