pub trait FromConfig<Entry>: Sized {
    fn from_config(entry: &Entry) -> Self;
}
