use galaxy_network::raw::Rights;

use super::{
    idpool::IdPool,
    proxy::Proxy,
};

pub struct User {
    pub proxy: Proxy,
    pub rights: Rights,
}

impl User {
    pub fn new(id_pool: IdPool, rights: Rights) -> Self {
        Self {
            rights,
            proxy: Proxy::new(id_pool),
        }
    }
}
