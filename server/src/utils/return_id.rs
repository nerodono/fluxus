use idpool::interface::IdPool;
use tokio::sync::Mutex;

#[must_use = "ID must be returned back to the pool or explicitly ignored"]
pub struct ReturnId<T, V> {
    id: T,
    value: V,
}

impl<T, V> ReturnId<T, V> {
    pub async fn return_id<P: IdPool<Id = T>>(self, pool: &Mutex<P>) -> V {
        pool.lock().await.return_id(self.id);
        self.value
    }

    pub const fn new(id: T, value: V) -> Self {
        Self { id, value }
    }
}
