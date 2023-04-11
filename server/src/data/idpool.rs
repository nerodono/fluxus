use std::sync::Arc;

use idpool::flat::FlatIdPool;
use tokio::sync::Mutex;

type IdPoolImpl = FlatIdPool<u16>;
pub type IdPool = Arc<Mutex<IdPoolImpl>>;

pub fn create_id_pool() -> IdPool {
    IdPool::new(IdPoolImpl::new(0).into())
}
