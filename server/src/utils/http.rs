pub fn is_valid_host(buf: &[u8]) -> bool {
    if buf.starts_with(b".") || buf.ends_with(b".") {
        false
    } else {
        buf.iter().all(u8::is_ascii)
    }
}
