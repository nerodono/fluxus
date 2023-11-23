use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct InfoPayload<'a> {
    pub server_name: Cow<'a, str>,
}
