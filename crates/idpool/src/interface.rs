pub trait IdPool: Send {
    /// Type of ID in the pool
    type Id;

    /// Request ID from the ``IDPool``.
    ///
    /// Returns [`None`] if next ID would overflow
    /// underlying type.
    fn request(&mut self) -> Option<Self::Id>;

    /// Return ID back to the ID pool.
    fn return_id(&mut self, id: Self::Id);
}
