use integral_enum::integral_enum;

#[derive(Debug)]
pub enum Body {
    Chunked,
    ContentLength(usize),
}

#[integral_enum]
pub enum State {
    RequestLine,
    Header,
}
