use cfg_if::cfg_if;

use crate::error::NonCriticalResult;

cfg_if! {
    if #[cfg(feature = "http")] {
        use crate::features::http::HttpFeature;
    }
}

#[derive(Clone)]
pub struct FeatureGate {
    #[cfg(feature = "http")]
    http: HttpFeature,
}

impl FeatureGate {
    #[cfg(feature = "http")]
    pub const fn http(&self) -> NonCriticalResult<&HttpFeature> {
        Ok(&self.http)
    }

    #[cfg(not(feature = "http"))]
    pub const fn http<T>(&self) -> NonCriticalResult<T> {
        use crate::error::NonCriticalError;

        Err(NonCriticalError::FeatureIsDisabled)
    }

    pub const fn new(#[cfg(feature = "http")] http: HttpFeature) -> Self {
        Self {
            #[cfg(feature = "http")]
            http,
        }
    }
}
