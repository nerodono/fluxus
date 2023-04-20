use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{
            AtomicBool,
            Ordering,
        },
        Arc,
    },
};

use http_body_util::Full;
use hyper::{
    body::{
        Bytes,
        Incoming,
    },
    http::HeaderValue,
    service::Service,
    HeaderMap,
    Request,
    Response,
};

use crate::{
    config::HttpDiscoveryMethod,
    features::http::storage::HttpStorage,
};

pub struct HttpProxyService {
    discovery_method: HttpDiscoveryMethod,
    storage: Arc<HttpStorage>,
    id: Option<u16>,
    reset: Arc<AtomicBool>,
}

impl Service<Request<Incoming>> for HttpProxyService {
    type Error = hyper::Error;
    type Future = Pin<
        Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;
    type Response = Response<Full<Bytes>>;

    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        #[inline]
        fn collect_headers(
            headers: &HeaderMap<HeaderValue>,
        ) -> Vec<(String, HeaderValue)> {
            headers
                .into_iter()
                .map(|(name, value)| (name.to_string(), value.clone()))
                .collect()
        }
        if self.reset.load(Ordering::Acquire) {
            self.id = None;
            self.reset.store(false, Ordering::Release);
        }

        let storage = Arc::clone(&self.storage);
        let reset_flag = Arc::clone(&self.reset);

        let future = async move {
            {
                match self.discovery_method {
                    HttpDiscoveryMethod::Path => {
                        let path = req.uri().path();
                        if !path.starts_with('/') {
                            return mk_static_str_response(
                                "Invalid path",
                                400,
                            );
                        }

                        let path_without_sep = &path[1..];
                        let (fragment, tail) = path_without_sep
                            .split_once('/')
                            .unwrap_or((path_without_sep, ""));
                        let endpoints = storage.raw_endpoints().read().await;
                        let Some(endpoint) = endpoints.get(fragment) else {
                            reset_flag.store(true, Ordering::Release);
                            return mk_static_str_response("Endpoint not found", 404);
                        };
                    }
                }
            }

            mk_static_str_response("TODO", 200)
        };

        Box::pin(future)
    }
}

#[allow(clippy::unnecessary_wraps)]
fn mk_static_str_response<E>(
    plain: &'static str,
    code: u16,
) -> Result<Response<Full<Bytes>>, E> {
    Response::builder()
        .status(code)
        .header("Content-Type", "text/plain")
        .body(Full::new(Bytes::from_static(plain.as_bytes())))
        .map_err(|_| panic!())
}
