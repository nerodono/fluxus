#[cold]
pub const fn cold_fn() {}

pub const fn likely(val: bool) -> bool {
    if val {
        true
    } else {
        cold_fn();
        false
    }
}

pub const fn unlikely(val: bool) -> bool {
    if val {
        cold_fn();
        true
    } else {
        false
    }
}
