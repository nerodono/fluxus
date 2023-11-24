pub struct MasterServerWriter<W> {
    writer: W,
}

impl<W> MasterServerWriter<W> {
    pub const fn new(writer: W) -> Self {
        Self { writer }
    }
}
