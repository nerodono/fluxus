pub struct MasterClientReader<'a, R> {
    pub(crate) reader: &'a mut R,
}
