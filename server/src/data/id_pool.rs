use std::sync::Arc;

use idpool::flat::FlatIdPool;
use tokio::sync::Mutex;

pub type RawIdPool = FlatIdPool<u16>;
pub type IdPoolImpl = Arc<Mutex<RawIdPool>>;

#[inline]
pub fn create_id_pool() -> IdPoolImpl {
    IdPoolImpl::new(RawIdPool::new(0).into())
}

#[inline]
pub fn clone_id_pool(pool: &IdPoolImpl) -> IdPoolImpl {
    Arc::clone(pool)
}
