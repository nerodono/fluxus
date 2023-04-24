use const_format::formatcp;

const NEOGROK_VER: &str = formatcp!("Neogrok/{}", env!("CARGO_PKG_VERSION"));

const HTTP_BAD_REQ_BODY: &str = formatcp!("Bad request\n{NEOGROK_VER}");
const HTTP_NOT_FOUND_BODY: &str = formatcp!("Not found\r\n{NEOGROK_VER}");

pub const HTTP_NOT_FOUND: &str = formatcp!(
    "HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Type: \
     text/plain\r\nServer: {NEOGROK_VER}\r\nContent-Length: \
     {body_len}\r\n\r\n{body}",
    body_len = HTTP_NOT_FOUND_BODY.len(),
    body = HTTP_NOT_FOUND_BODY
);
pub const HTTP_BAD_REQUEST: &str = formatcp!(
    "HTTP/1.1 400 Bad Request\r\nConnection: close\r\nContent-Type: \
     text/plain\r\nServer: Neogrok/{version}\r\nContent-Length: \
     {body_len}\r\n\r\n{body}",
    version = env!("CARGO_PKG_VERSION"),
    body = HTTP_BAD_REQ_BODY,
    body_len = HTTP_BAD_REQ_BODY.len()
);
