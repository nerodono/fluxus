use num::{
    Bounded,
    Integer,
};

use crate::interface::IdPool;

/// Flat id pool. Works like FIFO queue for id requesting if
/// it has some returned IDs.
pub struct FlatIdPool<I> {
    vec: Vec<I>,
    last: I,
}

impl<I> IdPool for FlatIdPool<I>
where
    I: Copy + Send + Integer + Bounded,
{
    type Id = I;

    fn request(&mut self) -> Option<Self::Id> {
        if let Some(id) = self.vec.pop() {
            Some(id)
        } else if I::max_value() == self.last {
            None
        } else {
            let prev = self.last;
            self.last = prev + I::one();
            Some(prev)
        }
    }

    fn return_id(&mut self, id: Self::Id) {
        self.vec.push(id);
    }
}

impl<I> FlatIdPool<I> {
    pub const fn new(start: I) -> Self {
        Self {
            vec: Vec::new(),
            last: start,
        }
    }
}
