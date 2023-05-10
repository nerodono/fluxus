use idpool::interface::IdPool;
use tokio::sync::mpsc;

use crate::{
    data::{
        commands::{
            http::{
                HttpMasterCommand,
                HttpSlaveCommand,
            },
            master::HttpPermit,
        },
        proxy::Pool,
    },
    error::{
        HttpError,
        HttpResult,
    },
};

pub struct Endpoint {
    pool: Pool,
    permit: HttpPermit,
}

impl Endpoint {
    pub async fn assign_id(
        &self,
        chan: mpsc::Sender<HttpSlaveCommand>,
    ) -> HttpResult<(u16, HttpPermit)> {
        let id = self
            .pool
            .lock()
            .await
            .request()
            .ok_or(HttpError::PoolExhausted)?;
        self.permit
            .send(HttpMasterCommand::Connected { chan }.identified(id))
            .await?;
        Ok((id, self.permit.clone()))
    }

    pub const fn new(pool: Pool, permit: HttpPermit) -> Self {
        Self { pool, permit }
    }
}
