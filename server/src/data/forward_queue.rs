use std::ops::Range;

#[derive(Debug)]
pub struct ForwardQueue {
    pub request_line: Option<Range<usize>>,
    pub headers_len: usize,
}

impl ForwardQueue {
    #[track_caller]
    pub fn range(&self) -> Range<usize> {
        let request_line = self.request_line.clone().unwrap();
        let start = request_line.start;

        start..start + self.headers_len
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
