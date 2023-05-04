use std::{
    ops::Deref,
    sync::Arc,
};

use cfg_if::cfg_if;

#[allow(unused_imports)]
use crate::error::NonCriticalError;
use crate::error::NonCriticalResult;

cfg_if! {
    if #[cfg(feature = "http")] {
        use super::features::http::HttpChannel;
    }
}

pub struct FeatureGateRef {
    #[cfg(feature = "http")]
    http: HttpChannel,
}

#[derive(Clone)]
pub struct FeatureGate {
    ref_: Arc<FeatureGateRef>,
}

impl Deref for FeatureGate {
    type Target = FeatureGateRef;

    fn deref(&self) -> &Self::Target {
        &self.ref_
    }
}

#[allow(clippy::new_without_default)]
impl FeatureGate {
    pub fn new(#[cfg(feature = "http")] http: HttpChannel) -> Self {
        Self {
            ref_: Arc::new(FeatureGateRef {
                #[cfg(feature = "http")]
                http,
            }),
        }
    }
}

impl FeatureGateRef {
    cfg_if! {
        if #[cfg(feature = "http")] {
            pub const fn http(&self) -> NonCriticalResult<&HttpChannel> {
                Ok(&self.http)
            }
        } else {
            pub const fn http<T>(&self) -> NonCriticalResult<T> {
                Err(NonCriticalError::FeatureIsUnavailable("http"))
            }
        }
    }
}
