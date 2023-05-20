use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("ring is full")]
pub struct RingIsFull(pub Vec<u8>);

pub struct BufferRing<const N: usize> {
    ring: heapless::Vec<Vec<u8>, N>,
}

impl<const N: usize> BufferRing<N> {
    #[inline]
    pub fn try_pop(&mut self) -> Option<Vec<u8>> {
        self.ring.pop()
    }

    #[inline]
    pub fn push(&mut self, buffer: Vec<u8>) -> Result<(), RingIsFull> {
        self.ring.push(buffer).map_err(RingIsFull)
    }

    pub const fn new() -> Self {
        Self {
            ring: heapless::Vec::new(),
        }
    }
}
