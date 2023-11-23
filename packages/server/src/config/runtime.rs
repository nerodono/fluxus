use std::num::NonZeroUsize;

entity! {
    #[derive(Default)]
    struct RuntimeConfig {
        threads: Option<NonZeroUsize>
    }
}
