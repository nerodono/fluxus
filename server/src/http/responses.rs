#![allow(dead_code)]
use const_format::formatcp;

pub const SERVER_NAME: &str = env!("CARGO_PKG_NAME");
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SERVER_DISPLAY: &str = formatcp!("{SERVER_NAME}/{SERVER_VERSION}");

const NOT_FOUND_BODY: &str = "Not found\n";
const EXHAUSTED_BODY: &str =
    "The request is too long to fit in allocated buffer\n";

pub const INTERNAL_ERROR_TPL: &str = formatcp!(concat!(
    "HTTP/1.1 500 Internal Server Error\r\n",
    "Connection: close\r\n",
    "Server: {SERVER_DISPLAY}\r\n",
));
pub const BUFFER_EXHAUSTED: &str = formatcp!(
    concat!(
        "HTTP/1.1 500 Internal Server Error\r\n",
        "Connection: close\r\n",
        "Content-Length: {len}\r\n",
        "Server: {SERVER_DISPLAY}\r\n",
        "\r\n",
        "{body}"
    ),
    len = EXHAUSTED_BODY.len(),
    body = EXHAUSTED_BODY
);

pub const NOT_FOUND: &str = formatcp!(
    concat!(
        "HTTP/1.1 404 Not Found\r\n",
        "Connection: keep-alive\r\n",
        "Content-Length: {len}\r\n",
        "Server: {SERVER_DISPLAY}\r\n",
        "\r\n",
        "{body}"
    ),
    len = NOT_FOUND_BODY.len(),
    body = NOT_FOUND_BODY
);
