use galaxy_network::raw::Rights;

pub struct User {
    pub rights: Rights,
}

impl User {
    pub const fn new(rights: Rights) -> Self {
        Self { rights }
    }
}
