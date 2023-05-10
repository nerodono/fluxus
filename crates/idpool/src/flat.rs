use std::{
    mem,
    time::{
        Duration,
        Instant,
    },
};

use num::{
    Bounded,
    Integer,
};

use crate::interface::IdPool;

/// Flat id pool. Works like FIFO queue for id requesting if
/// it has some returned IDs.
pub struct FlatIdPool<I> {
    vec: Vec<I>,
    swap: Vec<I>,
    last: I,

    next_swap: Instant,
}

impl<I> IdPool for FlatIdPool<I>
where
    I: Copy + Send + Integer + Bounded,
{
    type Id = I;

    fn request(&mut self) -> Option<Self::Id> {
        let cur_time = Instant::now();
        if cur_time >= self.next_swap {
            self.swap.extend(self.vec.iter().copied());
            self.vec.clear();
            mem::swap(&mut self.vec, &mut self.swap);

            self.next_swap = Self::next_swap(cur_time);
        }

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
        self.swap.push(id);
    }
}

impl<I> FlatIdPool<I> {
    fn next_swap(now: Instant) -> Instant {
        now + Duration::from_secs(2)
    }

    pub fn new(start: I) -> Self {
        Self {
            vec: Vec::new(),
            swap: Vec::new(),
            last: start,
            next_swap: Self::next_swap(Instant::now()),
        }
    }
}
