use crate::error;

/// Pool of ids for the clients.
#[derive(Debug, Clone)]
pub struct FlatIdPool {
    mem: Vec<u16>,
    last: u16,
}

impl FlatIdPool {
    /// Push id to the pool. Prefixed with `_unchecked`
    /// because it will not check id presence in the pool.
    pub fn push_back_unchecked(&mut self, id: u16) {
        self.mem.push(id);
    }

    /// Requests id from the pool.
    pub fn request(&mut self) -> Result<u16, error::IdRequestError> {
        if let Some(id) = self.mem.pop() {
            Ok(id)
        } else {
            match self.last.overflowing_add(1) {
                (_, true) => Err(error::IdRequestError::Exceeded),
                (n, false) => {
                    let prev = self.last;
                    self.last = n;
                    Ok(prev)
                }
            }
        }
    }

    /// Creates pool with default
    pub const fn new() -> Self {
        Self {
            mem: Vec::new(),
            last: 0,
        }
    }
}
