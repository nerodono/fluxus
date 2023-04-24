use std::{
    collections::HashMap,
    mem,
};

use rustc_hash::FxHashMap;

use crate::{
    data::commands::{
        base::HttpPermit,
        http::GlobalHttpCommand,
    },
    features::http::HttpFeature,
};

pub struct HttpServer {
    domain_or_path: Option<Vec<u8>>,
    clients: FxHashMap<u16, HttpPermit>,
    feature: HttpFeature,
    started: bool,
}

impl HttpServer {
    pub fn new(domain_or_path: Vec<u8>, feature: HttpFeature) -> Self {
        Self {
            domain_or_path: Some(domain_or_path),
            feature,
            clients: HashMap::default(),
            started: false,
        }
    }
}

impl HttpServer {
    pub fn started(&mut self) {
        self.started = true;
    }

    fn domain_or_path(&self) -> &[u8] {
        unsafe { self.domain_or_path.as_ref().unwrap_unchecked() }
    }

    fn take_domain_or_path_out(&mut self) -> Vec<u8> {
        let domain_or_path =
            unsafe { mem::take(&mut self.domain_or_path).unwrap_unchecked() };
        self.domain_or_path = Some(Vec::new());
        domain_or_path
    }
}

impl Drop for HttpServer {
    fn drop(&mut self) {
        let domain_or_path = self.take_domain_or_path_out();
        _ = self
            .feature
            .send_command(GlobalHttpCommand::Unbind { domain_or_path });
    }
}
