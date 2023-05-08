use std::ops::Range;

#[derive(Debug)]
pub struct ForwardQueue {
    pub request_line: Option<Range<usize>>,
    pub headers_len: usize,
}

impl ForwardQueue {
    #[track_caller]
    #[allow(clippy::range_plus_one)]
    pub fn range(&self) -> Range<usize> {
        let request_line = self.request_line.clone().unwrap();
        let start = request_line.start;

        start..(request_line.end + self.headers_len + 1)
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn append_header(&mut self, of_len: usize) {
        self.headers_len += of_len;
    }

    pub fn fill_request_line(&mut self, line: Range<usize>) {
        self.request_line = Some(line);
    }

    pub const fn new() -> Self {
        Self {
            request_line: None,
            headers_len: 0,
        }
    }
}
